#include "stdafx.h"

#include "networkadaptermonitor.h"
#include <libcommon/error.h>
#include <libcommon/memory.h>
#include <sstream>


NetworkAdapterMonitor::NetworkAdapterMonitor(
	std::shared_ptr<common::logging::ILogSink> logSink,
	UpdateSinkType updateSink,
	FilterType filter
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
		const auto pair = m_adapters.emplace(
			table->Table[i].InterfaceLuid.Value,
			AdapterElement(
				table->Table[i],
				false,
				false
			)
		);

		if (m_filter(pair.first->second.adapter))
		{
			//addFilteredIfUnique(pair.first->second.adapter);
			m_filteredAdapters.push_back(pair.first->second.adapter);
			m_updateSink(
				pair.first->second.adapter,
				UpdateType::Add
			);
		}
	}

	const auto statusCb = NotifyIpInterfaceChange(AF_UNSPEC, Callback, this, FALSE, &m_notificationHandle);
	THROW_UNLESS(NO_ERROR, statusCb, "Register interface change notification");
}

NetworkAdapterMonitor::~NetworkAdapterMonitor()
{
	CancelMibChangeNotify2(m_notificationHandle);
}

const std::vector<MIB_IF_ROW2>& NetworkAdapterMonitor::getFilteredAdapters() const
{
	return m_filteredAdapters;
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

void NetworkAdapterMonitor::addFilteredIfUnique(const MIB_IF_ROW2 &adapter)
{
	auto filteredIt = std::find_if(m_filteredAdapters.begin(), m_filteredAdapters.end(), [&adapter](const MIB_IF_ROW2 &elem)
	{
		return elem.InterfaceLuid.Value == adapter.InterfaceLuid.Value;
	});

	if (m_filteredAdapters.end() == filteredIt)
	{
		m_filteredAdapters.push_back(adapter);
	}
}

void NetworkAdapterMonitor::addInternal(
	const MIB_IF_ROW2 &newIface,
	bool isIPv4,
	bool isIPv6
)
{
	const auto pair = m_adapters.emplace(
		newIface.InterfaceLuid.Value,
		AdapterElement(
			newIface,
			isIPv4,
			isIPv6
		)
	);

	if (m_filter(pair.first->second.adapter))
	{
		addFilteredIfUnique(pair.first->second.adapter);

		m_updateSink(
			pair.first->second.adapter,
			UpdateType::Add
		);
	}
}

void NetworkAdapterMonitor::callback(const MIB_IPINTERFACE_ROW *hint, MIB_NOTIFICATION_TYPE updateType)
{
	std::scoped_lock<std::mutex> processingLock(m_processingMutex);

	MIB_IF_ROW2 newIface = { 0 };
	newIface.InterfaceLuid = hint->InterfaceLuid;
	const auto status = GetIfEntry2(&newIface);

	if (NO_ERROR != status)
	{
		std::stringstream ss;

		ss << "GetIfEntry2() failed for LUID 0x" << std::hex << newIface.InterfaceLuid.Value
			<< " in NetworkAdapterMonitor::callback(), error: 0x" << status;

		throw std::runtime_error(ss.str());
	}

	bool isIPv4 = hint->Family == AF_INET;
	bool isIPv6 = hint->Family == AF_INET6;

	if (!isIPv4 && !isIPv6)
	{
		std::stringstream ss;

		ss << "Expected either AF_INET or AF_INET6 for LUID 0x"
			<< std::hex << newIface.InterfaceLuid.Value;

		throw std::runtime_error(ss.str());
	}

	switch (updateType)
	{
		case MibAddInstance:
		{
			const auto adapterIt = m_adapters.find(hint->InterfaceLuid.Value);

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
			}
			else
			{
				addInternal(newIface, isIPv4, isIPv6);
			}

			break;
		}
		case MibParameterNotification:
		{
			const auto adapterIt = m_adapters.find(hint->InterfaceLuid.Value);

			if (m_adapters.end() != adapterIt)
			{
				// update row
				MIB_IF_ROW2 &iface = adapterIt->second.adapter;
				iface = newIface;

				if (m_filter(iface))
				{
					//
					// "Update" is reported even if "Add" was not.
					//

					addFilteredIfUnique(iface);

					m_updateSink(
						iface,
						UpdateType::Update
					);
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
					m_adapters.erase(adapterIt);

					MIB_IF_ROW2 &iface = adapterIt->second.adapter;

					auto filteredIt = std::find_if(m_filteredAdapters.begin(), m_filteredAdapters.end(), [hint](const MIB_IF_ROW2 &elem)
					{
						return elem.InterfaceLuid.Value == hint->InterfaceLuid.Value;
					});

					//
					// Delete it here so that Add is reported if filter() returns false
					// here and true later.
					//
					// "Delete" is reported even if "Add" was not.
					//
					m_filteredAdapters.erase(filteredIt);

					if (m_filter(iface))
					{
						//m_filteredAdapters.erase(filteredIt);
						m_updateSink(
							iface,
							UpdateType::Delete
						);
					}
				}
			}

			break;
		}
	}
}
