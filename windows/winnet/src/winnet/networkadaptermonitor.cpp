#include "stdafx.h"

#include "networkadaptermonitor.h"
#include <libcommon/memory.h>
#include <sstream>

using namespace std::placeholders;


namespace
{

bool HasIPv4Interface(NET_LUID luid)
{
	MIB_IPINTERFACE_ROW iprow = { 0 };
	iprow.InterfaceLuid = luid;
	iprow.Family = AF_INET;

	const auto status = GetIpInterfaceEntry(&iprow);

	if (NO_ERROR == status)
	{
		return true;
	}
	else if (ERROR_NOT_FOUND != status)
	{
		common::error::Throw("Resolve IPv4 interface", status);
	}
	return false;
}

bool HasIPv6Interface(NET_LUID luid)
{
	MIB_IPINTERFACE_ROW iprow = { 0 };
	iprow.InterfaceLuid = luid;
	iprow.Family = AF_INET6;

	const auto status = GetIpInterfaceEntry(&iprow);

	if (NO_ERROR == status)
	{
		return true;
	}
	else if (ERROR_NOT_FOUND != status)
	{
		common::error::Throw("Resolve IPv6 interface", status);
	}
	return false;
}

}


NetworkAdapterMonitor::NetworkAdapterMonitor(
	std::shared_ptr<common::logging::ILogSink> logSink,
	UpdateSinkType updateSink,
	FilterType filter,
	std::shared_ptr<WinNotifier> notifier
)
	: m_logSink(m_logSink)
	, m_updateSink(updateSink)
	, m_filter(filter)
	, m_winNotifier(notifier)
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
		bool ipv4 = HasIPv4Interface(table->Table[i].InterfaceLuid);
		bool ipv6 = HasIPv6Interface(table->Table[i].InterfaceLuid);

		if (!ipv4 && !ipv6)
		{
			continue;
		}

		const auto pair = m_adapters.emplace(
			table->Table[i].InterfaceLuid.Value,
			AdapterElement(
				table->Table[i],
				ipv4,
				ipv6
			)
		);

		if (m_filter(pair.first->second.adapter))
		{
			m_filteredAdapters.push_back(pair.first->second.adapter);
		}
	}

	if (!m_filteredAdapters.empty())
	{
		m_updateSink(m_filteredAdapters, nullptr, UpdateType::Add);
	}

	m_winNotifier->attach(m_logSink, std::bind(&NetworkAdapterMonitor::callback, this, _1, _2));
}

NetworkAdapterMonitor::~NetworkAdapterMonitor()
{
	m_winNotifier->detach();
}

std::vector<MIB_IF_ROW2>::iterator NetworkAdapterMonitor::findFilteredAdapter(const MIB_IF_ROW2 &adapter)
{
	return std::find_if(m_filteredAdapters.begin(), m_filteredAdapters.end(), [&adapter](const MIB_IF_ROW2 &elem)
	{
		return elem.InterfaceLuid.Value == adapter.InterfaceLuid.Value;
	});
}

void NetworkAdapterMonitor::getIfEntry(MIB_IF_ROW2 &rowOut, NET_LUID luid)
{
	rowOut.InterfaceLuid = luid;
	const auto status = GetIfEntry2(&rowOut);

	if (NO_ERROR == status)
		return;

	std::stringstream ss;

	ss << "GetIfEntry2() failed for LUID 0x" << std::hex << rowOut.InterfaceLuid.Value
		<< " in NetworkAdapterMonitor::getIfEntry(), error: 0x" << status;

	throw std::runtime_error(ss.str());
}

void NetworkAdapterMonitor::callback(const MIB_IPINTERFACE_ROW *hint, MIB_NOTIFICATION_TYPE updateType)
{
	std::scoped_lock<std::mutex> processingLock(m_processingMutex);

	bool isIPv4 = hint->Family == AF_INET;
	bool isIPv6 = hint->Family == AF_INET6;

	if (!isIPv4 && !isIPv6)
	{
		std::stringstream ss;

		ss << "Expected either AF_INET or AF_INET6 for LUID 0x"
			<< std::hex << hint->InterfaceLuid.Value;

		throw std::runtime_error(ss.str());
	}

	switch (updateType)
	{
		case MibAddInstance:
		{
			const auto adapterIt = m_adapters.find(hint->InterfaceLuid.Value);
			MIB_IF_ROW2 *row = nullptr;

			if (m_adapters.end() != adapterIt)
			{
				if (isIPv4)
				{
					adapterIt->second.IPv4 = true;
				}
				else
				{
					adapterIt->second.IPv6 = true;
				}
				row = &adapterIt->second.adapter;
			}
			else
			{
				MIB_IF_ROW2 entry;
				getIfEntry(entry, hint->InterfaceLuid);

				const auto pair = m_adapters.emplace(
					entry.InterfaceLuid.Value,
					AdapterElement(
						entry,
						isIPv4,
						isIPv6
					)
				);

				row = &pair.first->second.adapter;
			}

			if (m_filter(*row))
			{
				//
				// Report Add event if this is new
				//
				if (m_filteredAdapters.end() == findFilteredAdapter(*row))
				{
					m_filteredAdapters.push_back(*row);
					m_updateSink(m_filteredAdapters, row, UpdateType::Add);
				}
			}
			else
			{
				//
				// Synthesize a Delete event if we're not
				// interested in this adapter anymore.
				//
				const auto it = findFilteredAdapter(*row);
				if (m_filteredAdapters.end() != it)
				{
					m_filteredAdapters.erase(it);
					m_updateSink(m_filteredAdapters, row, UpdateType::Delete);
				}
			}

			break;
		}
		case MibParameterNotification:
		{
			const auto adapterIt = m_adapters.find(hint->InterfaceLuid.Value);

			if (m_adapters.end() != adapterIt)
			{
				//
				// Update row content
				//
				MIB_IF_ROW2 &iface = adapterIt->second.adapter;
				getIfEntry(iface, hint->InterfaceLuid);

				if (m_filter(iface))
				{
					if (m_filteredAdapters.end() == findFilteredAdapter(iface))
					{
						m_filteredAdapters.push_back(iface);

						//
						// Report Add if we hadn't seen this adapter before.
						//
						m_updateSink(
							m_filteredAdapters,
							&iface,
							UpdateType::Add
						);
					}
					else
					{
						m_updateSink(
							m_filteredAdapters,
							&iface,
							UpdateType::Update
						);
					}
				}
				else
				{
					//
					// No longer interested in this adapter.
					// Synthesize a Delete event.
					//
					const auto it = findFilteredAdapter(iface);
					if (m_filteredAdapters.end() != it)
					{
						m_filteredAdapters.erase(it);

						m_updateSink(
							m_filteredAdapters,
							&iface,
							UpdateType::Delete
						);
					}
				}
			}

			break;
		}
		case MibDeleteInstance:
		{
			const auto adapterIt = m_adapters.find(hint->InterfaceLuid.Value);

			if (m_adapters.end() != adapterIt)
			{
				if (isIPv4)
				{
					adapterIt->second.IPv4 = false;
				}
				else
				{
					adapterIt->second.IPv6 = false;
				}

				if (!adapterIt->second.IPv4 &&
					!adapterIt->second.IPv6)
				{
					MIB_IF_ROW2 iface = adapterIt->second.adapter;
					
					m_adapters.erase(adapterIt);

					auto filteredIt = std::find_if(m_filteredAdapters.begin(), m_filteredAdapters.end(), [hint](const MIB_IF_ROW2 &elem)
					{
						return elem.InterfaceLuid.Value == hint->InterfaceLuid.Value;
					});

					if (m_filteredAdapters.end() != filteredIt)
					{
						//
						// Delete it here so that Add is reported if filter() returns false
						// here and true later.
						//
						m_filteredAdapters.erase(filteredIt);

						//
						// "Delete" will be reported only if "Add" was reported first.
						//
						if (m_filter(iface))
						{
							m_updateSink(
								m_filteredAdapters,
								&iface,
								UpdateType::Delete
							);
						}
					}
				}
			}

			break;
		}
	}
}

//
// DefaultWinNotifier
//

NetworkAdapterMonitor::DefaultWinNotifier::DefaultWinNotifier() :
	m_attached(false)
{
}

NetworkAdapterMonitor::DefaultWinNotifier::~DefaultWinNotifier()
{
	detach();
}

//static
void __stdcall NetworkAdapterMonitor::DefaultWinNotifier::Callback(void *context, MIB_IPINTERFACE_ROW *hint, MIB_NOTIFICATION_TYPE updateType)
{
	auto inst = reinterpret_cast<NetworkAdapterMonitor::DefaultWinNotifier *>(context);

	try
	{
		inst->m_callback(hint, updateType);
	}
	catch (const std::exception &err)
	{
		inst->m_logSink->error(err.what());
	}
	catch (...)
	{
		inst->m_logSink->error("Unspecified error in NetworkAdapterMonitor::Callback()");
	}
}

void NetworkAdapterMonitor::DefaultWinNotifier::attach(
	std::shared_ptr<common::logging::ILogSink> logSink,
	AdapterUpdate callback
)
{
	if (!m_attached)
	{
		const auto statusCb = NotifyIpInterfaceChange(
			AF_UNSPEC,
			Callback,
			static_cast<void*>(this),
			FALSE,
			&m_notificationHandle
		);
		
		THROW_UNLESS(NO_ERROR, statusCb, "Register interface change notification");

		m_logSink = logSink;
		m_callback = callback;
		m_attached = true;
	}
}

void NetworkAdapterMonitor::DefaultWinNotifier::detach()
{
	if (m_attached)
	{
		CancelMibChangeNotify2(m_notificationHandle);
		m_attached = false;
	}
}
