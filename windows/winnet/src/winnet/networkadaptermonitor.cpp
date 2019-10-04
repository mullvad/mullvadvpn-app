#include "stdafx.h"

#include "networkadaptermonitor.h"
#include <libcommon/error.h>
#include <libcommon/memory.h>
#include <sstream>


NetworkAdapterMonitor::NetworkAdapterMonitor(
	std::shared_ptr<common::logging::ILogSink> logSink,
	std::function<void(const MIB_IF_ROW2 &adapter, UpdateType updateType)> updateSink
) :
	NetworkAdapterMonitor(logSink, updateSink, [](const MIB_IF_ROW2 &) -> bool { return true; })
{
}

NetworkAdapterMonitor::NetworkAdapterMonitor(
	std::shared_ptr<common::logging::ILogSink> logSink,
	std::function<void(const MIB_IF_ROW2 &adapter, UpdateType updateType)> updateSink,
	std::function<bool(const MIB_IF_ROW2 &adapter)> filter
) :
	m_logSink(m_logSink)
	, m_updateSink(updateSink)
	, m_filter(filter)
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
		if (m_filter(table->Table[i]))
		{
			addInternal(table->Table[i]);
		}
	}

	const auto statusCb = NotifyIpInterfaceChange(AF_UNSPEC, Callback, this, FALSE, &m_notificationHandle);
	THROW_UNLESS(NO_ERROR, statusCb, "Register interface change notification");
}

NetworkAdapterMonitor::~NetworkAdapterMonitor()
{
	CancelMibChangeNotify2(m_notificationHandle);
}

size_t NetworkAdapterMonitor::numAdapters() const
{
	return m_adapters.size();
}

//static
void __stdcall NetworkAdapterMonitor::Callback(void *context, MIB_IPINTERFACE_ROW *hint, MIB_NOTIFICATION_TYPE updateType)
{
	auto nis = reinterpret_cast<NetworkAdapterMonitor *>(context);

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
		nis->m_logSink->error("Unspecified error in NetworkAdapterMonitor::Callback()");
	}
}

void NetworkAdapterMonitor::callback(const MIB_IPINTERFACE_ROW *hint, MIB_NOTIFICATION_TYPE updateType)
{
	switch (updateType)
	{
		case MibAddInstance:
		{
			add(hint->InterfaceLuid);

			const auto adapterIt = m_adapters.find(hint->InterfaceLuid.Value);
			if (m_filter(adapterIt->second))
			{
				m_updateSink(
					adapterIt->second,
					UpdateType::Add
				);
			}

			break;
		}
		case MibParameterNotification:
		{
			update(hint->InterfaceLuid);

			const auto adapterIt = m_adapters.find(hint->InterfaceLuid.Value);
			if (m_filter(adapterIt->second))
			{
				m_updateSink(
					adapterIt->second,
					UpdateType::Update
				);
			}

			break;
		}
		case MibDeleteInstance:
		{
			const auto adapterIt = m_adapters.find(hint->InterfaceLuid.Value);

			if (m_adapters.end() != adapterIt)
			{
				if (m_filter(adapterIt->second))
				{
					m_updateSink(
						adapterIt->second,
						UpdateType::Delete
					);
				}

				remove(hint->InterfaceLuid);
			}

			break;
		}
	}
}

void NetworkAdapterMonitor::addInternal(const MIB_IF_ROW2 &iface)
{
	//
	// The reason for removing an existing entry is that enabling
	// an interface on the adapter might change the overall properties in the
	// "row" which is merely an abstraction over all interfaces.
	//

	m_adapters.erase(iface.InterfaceLuid.Value);

	m_adapters.insert(std::make_pair(
		iface.InterfaceLuid.Value,
		iface
	));
}

void NetworkAdapterMonitor::add(NET_LUID luid)
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

	addInternal(newIface);
}

void NetworkAdapterMonitor::remove(NET_LUID luid)
{
	m_adapters.erase(luid.Value);
}

void NetworkAdapterMonitor::update(NET_LUID luid)
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

	addInternal(newIface);
}
