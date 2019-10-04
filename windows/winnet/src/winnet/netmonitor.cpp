#include "stdafx.h"
#include "netmonitor.h"
#include <libcommon/error.h>
#include <libcommon/memory.h>
#include <libcommon/string.h>
#include <sstream>


NetMonitor::NetMonitor
(
	std::shared_ptr<common::logging::ILogSink> logSink,
	NetMonitor::Notifier notifier,
	bool &currentConnectivity
)
	: m_logSink(logSink)
	, m_notifier(notifier)
	, m_connected(false)
	, m_notificationHandle(nullptr)
{
	UpdateConnectivity();

	currentConnectivity = m_connected;

	const auto status = NotifyIpInterfaceChange(AF_UNSPEC, Callback, this, FALSE, &m_notificationHandle);

	THROW_UNLESS(NO_ERROR, status, "Register interface change notification");

	if (false == m_connected)
	{
		LogOfflineState();
	}
}

NetMonitor::~NetMonitor()
{
	CancelMibChangeNotify2(m_notificationHandle);
}

void NetMonitor::UpdateConnectivity()
{
	m_connected = m_netInterfaces.AnyConnected();
}

//static
void __stdcall NetMonitor::Callback(void *context, MIB_IPINTERFACE_ROW *hint, MIB_NOTIFICATION_TYPE updateType)
{
	auto nm = reinterpret_cast<NetMonitor *>(context);

	try
	{
		nm->callback(hint, updateType);
	}
	catch (const std::exception &err)
	{
		nm->m_logSink->error(err.what());
	}
	catch (...)
	{
		nm->m_logSink->error("Unspecified error in NetMonitor::Callback()");
	}
}

void NetMonitor::callback(MIB_IPINTERFACE_ROW *hint, MIB_NOTIFICATION_TYPE updateType)
{
	std::scoped_lock<std::mutex> processingLock(m_processingMutex);

	switch (updateType)
	{
		case MibAddInstance:
		{
			m_netInterfaces.Add(hint->InterfaceLuid);
			break;
		}
		case MibDeleteInstance:
		{
			m_netInterfaces.Remove(hint->InterfaceLuid);
			break;
		}
		case MibParameterNotification:
		{
			m_netInterfaces.Update(hint->InterfaceLuid);
			break;
		}
	}

	const auto previousConnectivity = m_connected;

	UpdateConnectivity();

	if (previousConnectivity != m_connected)
	{
		m_notifier(m_connected);

		if (false == m_connected)
		{
			LogOfflineState();
		}
	}
}

void NetMonitor::LogOfflineState()
{
	//
	// There is a race condition here because logging is not done using the
	// same data set that the online/offline logic processes.
	//
	// Not much of a problem really, this is temporary logging.
	//

	m_logSink->info("Machine is offline");

	MIB_IF_TABLE2 *table;

	const auto status = GetIfTable2(&table);

	if (NO_ERROR != status)
	{
		m_logSink->error("Failed to acquire list of network interfaces. Aborting detailed logging");
		return;
	}

	common::memory::ScopeDestructor sd;

	sd += [table]()
	{
		FreeMibTable(table);
	};

	m_logSink->info("Begin detailed listing of network interfaces");

	for (ULONG i = 0; i < table->NumEntries; ++i)
	{
		const auto &iface = table->Table[i];

		//
		// Don't flood the log with garbage.
		//
		const auto blacklist = std::vector<std::wstring>
		{
			L"WFP Native MAC Layer LightWeight Filter",
			L"QoS Packet Scheduler",
			L"WFP 802.3 MAC Layer LightWeight Filter",
			L"Microsoft Kernel Debug Network Adapter",
			L"Software Loopback Interface",
			L"Microsoft Teredo Tunneling Adapter",
			L"Microsoft IP-HTTPS Platform Adapter",
			L"Microsoft 6to4 Adapter",
			L"WAN Miniport",
			L"WiFi Filter Driver",
			L"Microsoft Wi-Fi Direct Virtual Adapter",
		};

		bool blacklisted = false;

		for (const auto &black : blacklist)
		{
			if (nullptr != wcsstr(iface.Description, black.c_str()))
			{
				blacklisted = true;
				break;
			}
		}

		if (blacklisted)
		{
			continue;
		}

		std::stringstream ss;

		ss << "Detailed interface logging" << std::endl;
		ss << "Interface ordinal " << i << std::endl;

		{
			const auto s = std::wstring(L"  Alias: ").append(iface.Alias);
			ss << common::string::ToAnsi(s) << std::endl;
		}

		{
			const auto s = std::wstring(L"  Description: ").append(iface.Description);
			ss << common::string::ToAnsi(s) << std::endl;
		}

		ss << "  PhysicalAddressLength: " << iface.PhysicalAddressLength << std::endl;
		ss << "  Type: " << iface.Type << std::endl;
		ss << "  MediaType: " << iface.MediaType << std::endl;
		ss << "  PhysicalMediumType: " << iface.PhysicalMediumType << std::endl;
		ss << "  AccessType: " << iface.AccessType << std::endl;

		//
		// Bool cast prevents idiot stream from inserting literal 0/1.
		//

		ss << "  InterfaceAndOperStatusFlags.HardwareInterface: " << (bool)iface.InterfaceAndOperStatusFlags.HardwareInterface << std::endl;
		ss << "  InterfaceAndOperStatusFlags.FilterInterface: " << (bool)iface.InterfaceAndOperStatusFlags.FilterInterface << std::endl;
		ss << "  InterfaceAndOperStatusFlags.ConnectorPresent: " << (bool)iface.InterfaceAndOperStatusFlags.ConnectorPresent << std::endl;
		ss << "  InterfaceAndOperStatusFlags.NotAuthenticated: " << (bool)iface.InterfaceAndOperStatusFlags.NotAuthenticated << std::endl;
		ss << "  InterfaceAndOperStatusFlags.NotMediaConnected: " << (bool)iface.InterfaceAndOperStatusFlags.NotMediaConnected << std::endl;
		ss << "  InterfaceAndOperStatusFlags.Paused: " << (bool)iface.InterfaceAndOperStatusFlags.Paused << std::endl;
		ss << "  InterfaceAndOperStatusFlags.LowPower: " << (bool)iface.InterfaceAndOperStatusFlags.LowPower << std::endl;
		ss << "  InterfaceAndOperStatusFlags.EndPointInterface: " << (bool)iface.InterfaceAndOperStatusFlags.EndPointInterface << std::endl;

		ss << "  OperStatus: " << iface.OperStatus << std::endl;
		ss << "  AdminStatus: " << iface.AdminStatus << std::endl;
		ss << "  MediaConnectState: " << iface.MediaConnectState << std::endl;
		ss << "  TransmitLinkSpeed: " << iface.TransmitLinkSpeed << std::endl;

		ss << "  ReceiveLinkSpeed: " << iface.ReceiveLinkSpeed << std::endl;
		ss << "  InUcastPkts:" << iface.InUcastPkts;

		m_logSink->info(ss.str().c_str());
	}

	m_logSink->info("End detailed listing of network interfaces");
}
