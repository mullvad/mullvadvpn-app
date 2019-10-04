#include "stdafx.h"

#include "NetworkInterfaceStatuses.h"
#include <libcommon/error.h>
#include <libcommon/memory.h>
#include <sstream>


NetworkInterfaceStatuses::NetworkInterfaceStatuses(
	std::shared_ptr<common::logging::ILogSink> logSink,
	std::function<bool(const MIB_IF_ROW2 &adapter)> filter,
	std::function<void(const MIB_IPINTERFACE_ROW *hint, bool connected)> updateSink
) :
	m_logSink(m_logSink),
	m_updateSink(updateSink)
{
	MIB_IF_TABLE2 *table;

	const auto status = GetIfTable2(&table);

	THROW_UNLESS(NO_ERROR, status, "Acquire network interface table");

	common::memory::ScopeDestructor sd;

	sd += [table]()
	{
		FreeMibTable(table);
	};

	for (ULONG i = 0; i < table->NumEntries; ++i)
	{
		if (filter(table->Table[i]))
		{
			addInternal(table->Table[i]);
		}
	}

	const auto statusCb = NotifyIpInterfaceChange(AF_UNSPEC, Callback, this, FALSE, &m_notificationHandle);
	THROW_UNLESS(NO_ERROR, statusCb, "Register interface change notification");
}

NetworkInterfaceStatuses::~NetworkInterfaceStatuses()
{
	CancelMibChangeNotify2(m_notificationHandle);
}

//static
void __stdcall NetworkInterfaceStatuses::Callback(void *context, MIB_IPINTERFACE_ROW *hint, MIB_NOTIFICATION_TYPE updateType)
{
	auto nis = reinterpret_cast<NetworkInterfaceStatuses *>(context);

	try
	{
		nis->callback(hint, updateType);
	}
	catch (const std::exception &err)
	{
		nis->m_logSink->error(err.what());
	}
	catch (...)
	{
		nis->m_logSink->error("Unspecified error in NetworkInterfaceStatuses::Callback()");
	}
}

void NetworkInterfaceStatuses::callback(const MIB_IPINTERFACE_ROW *hint, MIB_NOTIFICATION_TYPE updateType)
{
	std::scoped_lock<std::mutex> processingLock(m_processingMutex);

	if (m_interfaces.find(hint->InterfaceLuid.Value) == m_interfaces.end())
	{
		return;
	}

	bool previousStatus;

	auto ifaceIter = m_interfaces.find(hint->InterfaceLuid.Value);
	if (ifaceIter != m_interfaces.end())
	{
		previousStatus = ifaceIter->second.connected;
	}
	else
	{
		previousStatus = false;
	}

	switch (updateType)
	{
		case MibAddInstance:
		{
			add(hint->InterfaceLuid);
			break;
		}
		case MibParameterNotification:
		{
			update(hint->InterfaceLuid);
			break;
		}

		case MibDeleteInstance:
		{
			remove(hint->InterfaceLuid);

			// disconnect
			m_updateSink(hint, false);

			return;
		}
	}

	if (ifaceIter == m_interfaces.end())
	{
		ifaceIter = m_interfaces.find(hint->InterfaceLuid.Value);
	}

	if (ifaceIter != m_interfaces.end())
	{
		if (ifaceIter->second.connected != previousStatus)
			m_updateSink(hint, ifaceIter->second.connected);
	}
}

bool NetworkInterfaceStatuses::connected(NET_LUID luid) const
{
	const auto& iface = m_interfaces.find(luid.Value);
	if (iface == m_interfaces.end())
	{
		throw std::runtime_error("Unknown network interface");
	}
	return iface->second.connected;
}

bool NetworkInterfaceStatuses::anyConnected() const
{
	for (const auto niIter : m_interfaces)
	{
		const auto entry = niIter.second;

		if (entry.connected)
		{
			return true;
		}
	}

	return false;
}

void NetworkInterfaceStatuses::addInternal(const MIB_IF_ROW2 &iface)
{
	bool connected = (
		NET_IF_ADMIN_STATUS_UP == iface.AdminStatus
		&& IfOperStatusUp == iface.OperStatus
		&& MediaConnectStateConnected == iface.MediaConnectState
	);

	Entry e(
		iface.InterfaceLuid.Value,
		connected
	);
	m_interfaces.insert(std::make_pair(e.luid, e));
}

void NetworkInterfaceStatuses::add(NET_LUID luid)
{
	MIB_IF_ROW2 newIface = { 0 };
	newIface.InterfaceLuid = luid;

	const auto status = GetIfEntry2(&newIface);

	if (NO_ERROR != status)
	{
		std::stringstream ss;

		ss << "GetIfEntry2() failed for LUID 0x" << std::hex << newIface.InterfaceLuid.Value
			<< " during processing of MibAddInstance, error: 0x" << status;

		throw std::runtime_error(ss.str());
	}

	//
	// The reason for removing an existing entry is that enabling
	// an interface on the adapter might change the overall properties in the
	// "row" which is merely an abstraction over all interfaces.
	//

	m_interfaces.erase(newIface.InterfaceLuid.Value);
	addInternal(newIface);
}

void NetworkInterfaceStatuses::remove(NET_LUID luid)
{
	m_interfaces.erase(luid.Value);

	MIB_IF_ROW2 newIface = { 0 };
	newIface.InterfaceLuid = luid;

	const auto status = GetIfEntry2(&newIface);

	if (NO_ERROR == status)
	{
		addInternal(newIface);
	}
}

void NetworkInterfaceStatuses::update(NET_LUID luid)
{
	MIB_IF_ROW2 newIface = { 0 };
	newIface.InterfaceLuid = luid;

	const auto status = GetIfEntry2(&newIface);

	if (NO_ERROR != status)
	{
		//
		// Only update the cache if we can look up the interface details.
		// This way, if the interface was connected and continues to be so, we don't
		// mistakenly switch the status to "offline".
		//

		std::stringstream ss;

		ss << "GetIfEntry2() failed for LUID 0x" << std::hex << newIface.InterfaceLuid.Value
			<< " during processing of MibParameterNotification, error: 0x" << status;

		throw std::runtime_error(ss.str());
	}

	m_interfaces.erase(newIface.InterfaceLuid.Value);
	addInternal(newIface);
}
