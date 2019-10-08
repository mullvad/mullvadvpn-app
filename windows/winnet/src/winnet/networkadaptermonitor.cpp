#include "stdafx.h"

#include "networkadaptermonitor.h"
#include <libcommon/error.h>
#include <libcommon/memory.h>
#include <sstream>


NetworkAdapterMonitor::NetworkAdapterMonitor(
	std::shared_ptr<common::logging::ILogSink> logSink,
	UpdateSink updateSink,
	Filter filter
)
	: m_logSink(m_logSink)
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

const std::map<ULONG64, NetworkAdapterMonitor::AdapterElement>& NetworkAdapterMonitor::getAdapters() const
{
	return m_adapters;
}

//static
void __stdcall NetworkAdapterMonitor::Callback(void *context, MIB_IPINTERFACE_ROW *hint, MIB_NOTIFICATION_TYPE updateType)
{
	auto nam = reinterpret_cast<NetworkAdapterMonitor *>(context);

	try
	{
		nam->callback(hint, updateType);
	}
	catch (const std::exception &err)
	{
		nam->m_logSink->error(err.what());
	}
	catch (...)
	{
		nam->m_logSink->error("Unspecified error in NetworkAdapterMonitor::Callback()");
	}
}

void NetworkAdapterMonitor::callback(const MIB_IPINTERFACE_ROW *hint, MIB_NOTIFICATION_TYPE updateType)
{
	std::scoped_lock<std::mutex> processingLock(m_processingMutex);

	switch (updateType)
	{
		case MibAddInstance:
		{
			add(hint->InterfaceLuid);

			const auto adapterIt = m_adapters.find(hint->InterfaceLuid.Value);
			if (m_filter(adapterIt->second.adapter))
			{
				m_updateSink(
					adapterIt->second.adapter,
					UpdateType::Add
				);
			}

			break;
		}
		case MibParameterNotification:
		{
			update(hint->InterfaceLuid);

			const auto adapterIt = m_adapters.find(hint->InterfaceLuid.Value);
			if (m_filter(adapterIt->second.adapter))
			{
				m_updateSink(
					adapterIt->second.adapter,
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
				if (m_filter(adapterIt->second.adapter))
				{
					m_updateSink(
						adapterIt->second.adapter,
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
	const auto elemIt = m_adapters.find(iface.InterfaceLuid.Value);
	if (elemIt != m_adapters.end())
	{
		elemIt->second.refcount++;
	}
	else
	{
		AdapterElement elem;
		elem.adapter = iface;
		m_adapters[iface.InterfaceLuid.Value] = elem;
	}
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
			<< " in NetworkAdapterMonitor::add(), error: 0x" << status;

		throw std::runtime_error(ss.str());
	}

	addInternal(newIface);
}

void NetworkAdapterMonitor::remove(NET_LUID luid)
{
	const auto elemIt = m_adapters.find(luid.Value);
	if (elemIt != m_adapters.end())
	{
		elemIt->second.refcount--;

		if (elemIt->second.refcount == 0)
		{
			m_adapters.erase(luid.Value);
		}
	}
}

void NetworkAdapterMonitor::update(NET_LUID luid)
{
	MIB_IF_ROW2 newIface = { 0 };
	newIface.InterfaceLuid = luid;

	const auto status = GetIfEntry2(&newIface);

	if (NO_ERROR != status)
	{
		std::stringstream ss;

		ss << "GetIfEntry2() failed for LUID 0x" << std::hex << newIface.InterfaceLuid.Value
			<< " in NetworkAdapterMonitor::update(), error: 0x" << status;

		throw std::runtime_error(ss.str());
	}

	// update row
	const auto elemIt = m_adapters.find(newIface.InterfaceLuid.Value);
	elemIt->second.adapter = newIface;
}
