#include "stdafx.h"
#include "dnsagent.h"
#include "registrypaths.h"
#include "netsh.h"
#include <libcommon/trace/xtrace.h>
#include <libcommon/error.h>
#include <process.h>
#include <algorithm>

DnsAgent::DnsAgent(Protocol protocol, INameServerSource *nameServerSource, IRecoverySink *recoverySink, ILogSink *logSink)
	: m_protocol(protocol)
	, m_nameServerSource(nameServerSource)
	, m_recoverySink(recoverySink)
	, m_logSink(logSink)
	, m_thread(nullptr)
	, m_shutdownEvent(nullptr)
{
	constructNameServerUpdateEvent();
	constructRootMonitor();

	startTrackingInterfaces(discoverInterfaces());
	updateRecoveryData();

	constructThread();
}

DnsAgent::~DnsAgent()
{
	SetEvent(m_shutdownEvent);
	WaitForSingleObject(m_thread, INFINITE);

	CloseHandle(m_shutdownEvent);
	CloseHandle(m_thread);

	m_nameServerSource->unsubscribe(m_serverSourceEvent);
	CloseHandle(m_serverSourceEvent);
}

void DnsAgent::constructNameServerUpdateEvent()
{
	m_serverSourceEvent = CreateEventW(nullptr, TRUE, FALSE, nullptr);

	THROW_GLE_IF(nullptr, m_serverSourceEvent, "Create name server subscription event");

	m_nameServerSource->subscribe(m_serverSourceEvent);
}

void DnsAgent::constructRootMonitor()
{
	m_rootMonitor = common::registry::Registry::MonitorKey(HKEY_LOCAL_MACHINE,
		RegistryPaths::InterfaceRoot(m_protocol), { common::registry::RegistryEventFlag::SubkeyChange });
}

void DnsAgent::constructThread()
{
	m_shutdownEvent = CreateEventW(nullptr, TRUE, FALSE, nullptr);

	THROW_GLE_IF(nullptr, m_shutdownEvent, "Create shutdown event");

	auto rawThreadHandle = _beginthreadex(nullptr, 0, &DnsAgent::ThreadEntry, this, 0, nullptr);

	if (0 == rawThreadHandle)
	{
		throw std::runtime_error("Could not create monitoring thread");
	}

	m_thread = reinterpret_cast<HANDLE>(rawThreadHandle);
}

//static
unsigned __stdcall DnsAgent::ThreadEntry(void *parameters)
{
	try
	{
		reinterpret_cast<DnsAgent *>(parameters)->thread();
	}
	catch (std::exception &err)
	{
		const char *what = err.what();

		reinterpret_cast<DnsAgent *>(parameters)->m_logSink->error
		(
			"Critical error in monitoring thread", &what, 1
		);
	}
	catch (...)
	{
		reinterpret_cast<DnsAgent *>(parameters)->m_logSink->error
		(
			"Unspecified critical error in monitoring thread"
		);
	}

	return 0;
}

void DnsAgent::thread()
{
	for (;;)
	{
		std::vector<HANDLE> waitHandles;

		//
		// Reserve enough space in the array to hold:
		//
		// Shutdown event handle
		// Name servers source update event
		// Monitor handle for interfaces root key
		// Monitor handles for all interfaces
		//
		waitHandles.reserve(3 + m_trackedInterfaces.size());

		const size_t shutdownEventIndex = 0;
		const size_t serverSourceEventIndex = 1;
		const size_t rootKeyEventIndex = 2;
		const size_t firstInterfaceIndex = 3;

		waitHandles.push_back(m_shutdownEvent);
		waitHandles.push_back(m_serverSourceEvent);
		waitHandles.push_back(m_rootMonitor->queueSingleEvent());

		for (auto &interfaceData : m_trackedInterfaces)
		{
			waitHandles.push_back(interfaceData.monitor->queueSingleEvent());
		}

		//
		// Wait for one or more events to become signalled.
		//

		const auto status = WaitForMultipleObjects(static_cast<DWORD>(waitHandles.size()), &waitHandles[0], FALSE, INFINITE);

		if (WAIT_FAILED == status)
		{
			m_logSink->error("Failed to wait on events. Restarting wait in 1 minute.");

			if (WAIT_OBJECT_0 == WaitForSingleObject(m_shutdownEvent, 1000 * 60))
			{
				break;
			}

			continue;
		}

		const size_t firstSignalledIndex = status - WAIT_OBJECT_0;

		if (firstSignalledIndex == shutdownEventIndex)
		{
			break;
		}

		if (firstSignalledIndex >= firstInterfaceIndex)
		{
			XTRACE(L"Interface event is signalled");

			const auto result = processInterfaceEvent(&waitHandles[firstInterfaceIndex],
				firstSignalledIndex - firstInterfaceIndex);

			if (ProcessingResult::TrackingUpdated == result)
			{
				updateRecoveryData();
			}

			continue;
		}

		//
		// We can't easily tell which events have been signalled.
		//

		const auto interfaceResult = processInterfaceEvent(&waitHandles[firstInterfaceIndex], 0);
		auto rootResult = ProcessingResult::Nop;

		if (WAIT_OBJECT_0 == WaitForSingleObject(waitHandles[rootKeyEventIndex], 0))
		{
			XTRACE(L"Interfaces root key event is signalled");

			rootResult = processRootKeyEvent();
		}

		if (ProcessingResult::TrackingUpdated == interfaceResult
			|| ProcessingResult::TrackingUpdated == rootResult)
		{
			updateRecoveryData();
		}

		if (WAIT_OBJECT_0 == WaitForSingleObject(waitHandles[serverSourceEventIndex], 0))
		{
			XTRACE(L"Server source update event is signalled");
			ResetEvent(m_serverSourceEvent);

			processServerSourceEvent();
		}
	}

	XTRACE(L"Thread is exiting");
}

void DnsAgent::processServerSourceEvent()
{
	//
	// Check actual interface settings to determine which interfaces
	// need to have their settings overridden.
	//
	// Do NOT update 'preservedSettings' on the tracking entries because
	// it would overwrite legitimate settings with the previously enforced settings.
	//

	std::vector<std::wstring> interfaces;
	interfaces.reserve(m_trackedInterfaces.size());

	std::transform(m_trackedInterfaces.begin(), m_trackedInterfaces.end(), std::back_inserter(interfaces), [](const InterfaceData &interfaceData)
	{
		return interfaceData.interfaceGuid;
	});

	const auto updatedSnaps = createSnaps(interfaces);
	const auto enforcedServers = m_nameServerSource->getNameServers(m_protocol);

	for (const auto snap : updatedSnaps)
	{
		if (snap.needsOverriding(enforcedServers))
		{
			setNameServers(snap.interfaceGuid(), enforcedServers);
		}
	}
}

DnsAgent::ProcessingResult DnsAgent::processRootKeyEvent()
{
	ProcessingResult result = ProcessingResult::Nop;

	std::vector<std::wstring> oldInterfaces;
	oldInterfaces.reserve(m_trackedInterfaces.size());

	std::transform(m_trackedInterfaces.begin(), m_trackedInterfaces.end(), std::back_inserter(oldInterfaces), [](const InterfaceData &interfaceData)
	{
		return interfaceData.interfaceGuid;
	});

	auto currentInterfaces = discoverInterfaces();

	std::sort(oldInterfaces.begin(), oldInterfaces.end());
	std::sort(currentInterfaces.begin(), currentInterfaces.end());

	//
	// Stop tracking interfaces that have been removed.
	//

	std::vector<std::wstring> removedInterfaces;

	std::set_difference(oldInterfaces.begin(), oldInterfaces.end(), currentInterfaces.begin(), currentInterfaces.end(),
		std::back_inserter(removedInterfaces));

	if (false == removedInterfaces.empty())
	{
		result = ProcessingResult::TrackingUpdated;
		stopTrackingInterfaces(removedInterfaces);
	}

	//
	// Start tracking new interfaces.
	//

	std::vector<std::wstring> newInterfaces;

	std::set_difference(currentInterfaces.begin(), currentInterfaces.end(), oldInterfaces.begin(), oldInterfaces.end(),
		std::back_inserter(newInterfaces));

	if (false == newInterfaces.empty())
	{
		result = ProcessingResult::TrackingUpdated;
		startTrackingInterfaces(newInterfaces);
	}

	return result;
}

DnsAgent::ProcessingResult DnsAgent::processInterfaceEvent(const HANDLE *interfaceEvents, size_t startIndex)
{
	ProcessingResult result = ProcessingResult::Nop;

	//
	// 'interfaceEvents' runs in parallel with 'm_trackedInterfaces'.
	//

	const auto enforcedNameServers = m_nameServerSource->getNameServers(m_protocol);

	for (size_t i = startIndex; i < m_trackedInterfaces.size(); ++i)
	{
		if (WAIT_TIMEOUT == WaitForSingleObject(interfaceEvents[i], 0))
		{
			continue;
		}

		auto &interface = m_trackedInterfaces[i];

		XTRACE(L"Processing event for interface ", interface.interfaceGuid);

		try
		{
			InterfaceSnap updatedSnap(m_protocol, interface.interfaceGuid);

			if (updatedSnap.needsOverriding(enforcedNameServers))
			{
				result = ProcessingResult::TrackingUpdated;

				interface.preservedSettings = std::move(updatedSnap);
				setNameServers(interface.interfaceGuid, enforcedNameServers);
			}
		}
		catch (std::exception &err)
		{
			const char *what = err.what();

			m_logSink->error("Could not fetch updated interface settings. Probably because the interface was removed.", &what, 1);

			continue;
		}
		catch (...)
		{
			m_logSink->error("Could not fetch updated interface settings. Probably because the interface was removed.");

			continue;
		}
	}

	return result;
}

std::vector<std::wstring> DnsAgent::discoverInterfaces()
{
	auto regKey = common::registry::Registry::OpenKey(HKEY_LOCAL_MACHINE, RegistryPaths::InterfaceRoot(m_protocol));

	std::vector<std::wstring> interfaces;

	interfaces.reserve(20);

	regKey->enumerateSubKeys([&interfaces](const std::wstring &keyName)
	{
		interfaces.push_back(keyName);
		return true;
	});

	return interfaces;
}

std::vector<InterfaceSnap> DnsAgent::createSnaps(const std::vector<std::wstring> &interfaces)
{
	std::vector<InterfaceSnap> snaps;

	snaps.reserve(interfaces.size());

	for (const auto &interface : interfaces)
	{
		snaps.emplace_back(m_protocol, interface);
	}

	return snaps;
}

void DnsAgent::setNameServers(const std::wstring &interfaceGuid, const std::vector<std::wstring> &enforcedServers)
{
	XTRACE(L"Overriding name servers for interface ", interfaceGuid);

	uint32_t interfaceIndex = 0;

	try
	{
		interfaceIndex = NetSh::ConvertInterfaceGuidToIndex(interfaceGuid);
	}
	catch (...)
	{
		//
		// The interface cannot be linked to a virtual or physical adapter.
		//

		XTRACE(L"Ignoring floating interface ", interfaceGuid);
		return;
	}

	if (Protocol::IPv4 == m_protocol)
	{
		NetSh::Instance().SetIpv4StaticDns(interfaceIndex, enforcedServers);
	}
	else
	{
		NetSh::Instance().SetIpv6StaticDns(interfaceIndex, enforcedServers);
	}
}

void DnsAgent::startTrackingInterfaces(const std::vector<std::wstring> &interfaces)
{
	const auto snaps = createSnaps(interfaces);

	//
	// Override configured name servers on all interfaces, as necessary.
	//

	const auto enforcedServers = m_nameServerSource->getNameServers(m_protocol);

	for (const auto &snap : snaps)
	{
		if (snap.needsOverriding(enforcedServers))
		{
			setNameServers(snap.interfaceGuid(), enforcedServers);
		}
	}

	//
	// Create a tracking record for each interface.
	//

	for (const auto &snap : snaps)
	{
		const auto interfaceGuid = snap.interfaceGuid();

		XTRACE(L"Creating tracking entry for interface ", interfaceGuid);

		m_trackedInterfaces.emplace_back(interfaceGuid, snap, std::make_unique<InterfaceMonitor>(m_protocol, interfaceGuid));
	}
}

void DnsAgent::stopTrackingInterfaces(const std::vector<std::wstring> &interfaces)
{
	for (const auto &interfaceGuid : interfaces)
	{
		auto iter = std::find_if(m_trackedInterfaces.begin(), m_trackedInterfaces.end(), [&interfaceGuid](const InterfaceData &candidate)
		{
			return candidate.interfaceGuid == interfaceGuid;
		});

		if (m_trackedInterfaces.end() == iter)
		{
			m_logSink->error("Request to stop tracking non-tracked interface, ignoring.");

			continue;
		}

		XTRACE(L"Cancel tracking of interface ", interfaceGuid);

		m_trackedInterfaces.erase(iter);
	}
}

void DnsAgent::updateRecoveryData()
{
	std::vector<InterfaceSnap> snaps;

	snaps.reserve(m_trackedInterfaces.size());

	std::transform(m_trackedInterfaces.begin(), m_trackedInterfaces.end(), std::back_inserter(snaps), [](const InterfaceData &interfaceData)
	{
		return interfaceData.preservedSettings;
	});

	m_recoverySink->preserveSnaps(m_protocol, snaps);
}
