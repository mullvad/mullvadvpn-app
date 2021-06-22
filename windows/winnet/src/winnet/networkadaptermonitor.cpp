#include "stdafx.h"

#include "networkadaptermonitor.h"
#include <libcommon/memory.h>
#include <sstream>
#include <cstring>

using namespace std::placeholders;


NetworkAdapterMonitor::NetworkAdapterMonitor(
	std::shared_ptr<common::logging::ILogSink> logSink,
	UpdateSinkType updateSink,
	FilterType filter,
	std::shared_ptr<IDataProvider> dataProvider
)
	: m_logSink(logSink)
	, m_notificationHandle(nullptr)
	, m_updateSink(updateSink)
	, m_filter(filter)
	, m_dataProvider(dataProvider)
{
	//
	// Initialize adapters
	//
	
	MIB_IF_TABLE2 *table;

	const auto status = m_dataProvider->getIfTable2(&table);

	if (NO_ERROR != status)
	{
		THROW_WINDOWS_ERROR(status, "Acquire network interface table");
	}

	common::memory::ScopeDestructor sd;

	sd += [this, table]()
	{
		m_dataProvider->freeMibTable(table);
	};

	for (ULONG i = 0; i < table->NumEntries; ++i)
	{
		m_adapters[table->Table[i].InterfaceLuid.Value] = table->Table[i];

		if (filter(table->Table[i]))
		{
			m_filteredAdapters.push_back(table->Table[i]);
		}
	}

	//
	// Send initial notification
	//

	if (m_filteredAdapters.empty())
	{
		m_updateSink(m_filteredAdapters, nullptr, UpdateType::Update);
	}
	else
	{
		m_updateSink(m_filteredAdapters, nullptr, UpdateType::Add);
	}

	//
	// Listen to adapter events
	//

	const auto statusCb = m_dataProvider->notifyIpInterfaceChange(
		AF_UNSPEC,
		Callback,
		this,
		FALSE,
		&m_notificationHandle
	);

	if (NO_ERROR != statusCb)
	{
		THROW_WINDOWS_ERROR(statusCb, "Register interface change notification");
	}
}

NetworkAdapterMonitor::NetworkAdapterMonitor(
	std::shared_ptr<common::logging::ILogSink> logSink
	, UpdateSinkType updateSink
	, FilterType filter
) : NetworkAdapterMonitor(logSink, updateSink, filter, std::make_shared<SystemDataProvider>())
{
}

NetworkAdapterMonitor::~NetworkAdapterMonitor()
{
	if (nullptr != m_notificationHandle)
	{
		m_dataProvider->cancelMibChangeNotify2(m_notificationHandle);
		m_notificationHandle = nullptr;
	}
}

bool NetworkAdapterMonitor::hasIPv4Interface(NET_LUID luid) const
{
	MIB_IPINTERFACE_ROW iprow = { 0 };
	iprow.InterfaceLuid = luid;
	iprow.Family = AF_INET;

	const auto status = m_dataProvider->getIpInterfaceEntry(&iprow);

	if (NO_ERROR == status)
	{
		return true;
	}
	else if (ERROR_NOT_FOUND != status)
	{
		THROW_WINDOWS_ERROR(status, "Resolve IPv4 interface");
	}

	return false;
}

bool NetworkAdapterMonitor::hasIPv6Interface(NET_LUID luid) const
{
	MIB_IPINTERFACE_ROW iprow = { 0 };
	iprow.InterfaceLuid = luid;
	iprow.Family = AF_INET6;

	const auto status = m_dataProvider->getIpInterfaceEntry(&iprow);

	if (NO_ERROR == status)
	{
		return true;
	}
	else if (ERROR_NOT_FOUND != status)
	{
		THROW_WINDOWS_ERROR(status, "Resolve IPv6 interface");
	}

	return false;
}

std::vector<MIB_IF_ROW2>::iterator NetworkAdapterMonitor::findFilteredAdapter(const NET_LUID adapter)
{
	return std::find_if(m_filteredAdapters.begin(), m_filteredAdapters.end(), [&adapter](const MIB_IF_ROW2 &elem)
	{
		return elem.InterfaceLuid.Value == adapter.Value;
	});
}

std::optional<MIB_IF_ROW2> NetworkAdapterMonitor::getAdapter(NET_LUID luid) const
{
	MIB_IF_ROW2 rowOut = {0};
	rowOut.InterfaceLuid = luid;
	const auto status = m_dataProvider->getIfEntry2(&rowOut);

	if (NO_ERROR == status)
	{
		return std::make_optional(rowOut);
	}
	if (ERROR_FILE_NOT_FOUND == status)
	{
		return std::nullopt;
	}

	std::stringstream ss;

	ss << "GetIfEntry2() failed for LUID 0x" << std::hex << rowOut.InterfaceLuid.Value;

	THROW_WINDOWS_ERROR(status, ss.str().c_str());
}

void NetworkAdapterMonitor::callback(const MIB_IPINTERFACE_ROW *hint, MIB_NOTIFICATION_TYPE)
{
	const auto ifaceOpt = getAdapter(hint->InterfaceLuid);
	
	bool adapterEnabled = ifaceOpt.has_value()
		&& NET_IF_ADMIN_STATUS_UP == ifaceOpt->AdminStatus
		&& (hasIPv4Interface(ifaceOpt->InterfaceLuid)
			|| hasIPv6Interface(ifaceOpt->InterfaceLuid));

	const auto adapterIt = m_adapters.find(hint->InterfaceLuid.Value);

	if (adapterEnabled)
	{
		const auto &iface = *ifaceOpt;

		//
		// Check if the adapter has been added or updated
		//

		bool fieldsChanged;
		
		if (m_adapters.end() == adapterIt)
		{
			const auto pair = m_adapters.emplace(
				iface.InterfaceLuid.Value,
				iface
			);
			fieldsChanged = true;
		}
		else
		{
			//
			// Only send an Update event if the fields have changed
			//
			fieldsChanged = std::memcmp(
				&adapterIt->second,
				&iface,
				sizeof(MIB_IF_ROW2)
			) != 0;

			// update stored adapter
			adapterIt->second = iface;
		}

		if (m_filter(iface))
		{
			//
			// Report Add event if this is new
			//
			if (m_filteredAdapters.end() == findFilteredAdapter(iface.InterfaceLuid))
			{
				m_filteredAdapters.push_back(iface);
				m_updateSink(m_filteredAdapters, &iface, UpdateType::Add);
			}
			else if (fieldsChanged)
			{
				m_updateSink(m_filteredAdapters, &iface, UpdateType::Update);
			}
		}
		else
		{
			//
			// Synthesize a Delete event if we're no longer interested
			// in this adapter
			//
			const auto filteredIt = findFilteredAdapter(iface.InterfaceLuid);

			if (m_filteredAdapters.end() != filteredIt)
			{
				m_filteredAdapters.erase(filteredIt);
				m_updateSink(
					m_filteredAdapters,
					&iface,
					UpdateType::Delete
				);
			}
		}
	}
	else
	{
		if (m_adapters.end() == adapterIt)
		{
			return;
		}
		
		//
		// Remove the adapter
		//

		m_adapters.erase(adapterIt);

		const auto filteredIt = findFilteredAdapter(hint->InterfaceLuid);

		if (m_filteredAdapters.end() != filteredIt)
		{
			const auto &iface = ifaceOpt.value_or(*filteredIt);

			m_filteredAdapters.erase(filteredIt);

			//
			// We report 'Delete' for any adapter that was
			// approved by the filter when reported.
			//
			m_updateSink(
				m_filteredAdapters,
				&iface,
				UpdateType::Delete
			);
		}
	}
}

//static
void __stdcall NetworkAdapterMonitor::Callback(void *context, MIB_IPINTERFACE_ROW *hint, MIB_NOTIFICATION_TYPE updateType)
{
	auto inst = reinterpret_cast<NetworkAdapterMonitor *>(context);

	//
	// Calls into this function are supposed to be serialized by Windows.
	// That's not true on Windows 10 :-(
	//
	// This can be easily reproduced by changing the callback to never return,
	// and observing more events being delivered.
	//

	std::scoped_lock<std::mutex> lock(inst->m_callbackLock);

	try
	{
		inst->callback(hint, updateType);
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

//
// SystemDataProvider
//

DWORD NetworkAdapterMonitor::SystemDataProvider::notifyIpInterfaceChange(
	ADDRESS_FAMILY Family,
	PIPINTERFACE_CHANGE_CALLBACK Callback,
	PVOID CallerContext,
	BOOLEAN InitialNotification,
	HANDLE *NotificationHandle
)
{
	return NotifyIpInterfaceChange(
		Family,
		Callback,
		CallerContext,
		InitialNotification,
		NotificationHandle
	);
}

DWORD NetworkAdapterMonitor::SystemDataProvider::cancelMibChangeNotify2(HANDLE NotificationHandle)
{
	return CancelMibChangeNotify2(NotificationHandle);
}

DWORD NetworkAdapterMonitor::SystemDataProvider::getIfEntry2(PMIB_IF_ROW2 Row)
{
	return GetIfEntry2(Row);
}

DWORD NetworkAdapterMonitor::SystemDataProvider::getIfTable2(PMIB_IF_TABLE2 *Table)
{
	return GetIfTable2(Table);
}

DWORD NetworkAdapterMonitor::SystemDataProvider::getIpInterfaceEntry(PMIB_IPINTERFACE_ROW Row)
{
	return GetIpInterfaceEntry(Row);
}

void NetworkAdapterMonitor::SystemDataProvider::freeMibTable(PVOID Memory)
{
	FreeMibTable(Memory);
}
