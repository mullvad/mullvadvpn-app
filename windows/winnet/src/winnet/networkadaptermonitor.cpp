#include "stdafx.h"

#include "networkadaptermonitor.h"
#include <libcommon/memory.h>
#include <sstream>
#include <cstring>

using namespace std::placeholders;


namespace
{

void initAdaptersDefault(std::map<ULONG64, MIB_IF_ROW2> &adaptersOut)
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
		const auto pair = adaptersOut.emplace(
			table->Table[i].InterfaceLuid.Value,
			table->Table[i]
		);
	}
}

}


NetworkAdapterMonitor::NetworkAdapterMonitor(
	std::shared_ptr<common::logging::ILogSink> logSink,
	UpdateSinkType updateSink,
	FilterType filter,
	std::shared_ptr<WinNotifier> notifier,
	std::function<void(std::map<ULONG64, MIB_IF_ROW2> &adaptersOut)> initAdapters
)
	: m_logSink(logSink)
	, m_updateSink(updateSink)
	, m_filter(filter)
	, m_winNotifier(notifier)
{
	initAdapters(m_adapters);

	for (auto it = m_adapters.begin(); it != m_adapters.end(); ++it)
	{
		if (filter(it->second))
		{
			m_filteredAdapters.push_back(it->second);
		}
	}

	if (!m_filteredAdapters.empty())
	{
		m_updateSink(m_filteredAdapters, nullptr, UpdateType::Add);
	}

	m_winNotifier->attach(m_logSink, std::bind(&NetworkAdapterMonitor::callback, this, _1, _2));
}

NetworkAdapterMonitor::NetworkAdapterMonitor(
	std::shared_ptr<common::logging::ILogSink> logSink,
	UpdateSinkType updateSink,
	FilterType filter,
	std::shared_ptr<WinNotifier> notifier
) : NetworkAdapterMonitor(logSink, updateSink, filter, notifier, initAdaptersDefault)
{
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

void NetworkAdapterMonitor::callback(const MIB_IPINTERFACE_ROW *hint, MIB_NOTIFICATION_TYPE)
{
	std::scoped_lock<std::mutex> processingLock(m_processingMutex);

	MIB_IF_ROW2 iface;
	getIfEntry(iface, hint->InterfaceLuid);

	const bool adapterAvailable = !(iface.OperStatus != IfOperStatusUp ||
		iface.MediaConnectState != MediaConnectStateConnected ||
		iface.AdminStatus != NET_IF_ADMIN_STATUS_UP);

	const auto adapterIt = m_adapters.find(hint->InterfaceLuid.Value);

	if (adapterAvailable)
	{
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
			if (m_filteredAdapters.end() == findFilteredAdapter(iface))
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
			auto filteredIt = std::find_if(m_filteredAdapters.begin(), m_filteredAdapters.end(), [&iface](const MIB_IF_ROW2 &elem)
			{
				return elem.InterfaceLuid.Value == iface.InterfaceLuid.Value;
			});

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

		auto filteredIt = std::find_if(m_filteredAdapters.begin(), m_filteredAdapters.end(), [&iface](const MIB_IF_ROW2 &elem)
		{
			return elem.InterfaceLuid.Value == iface.InterfaceLuid.Value;
		});

		if (m_filteredAdapters.end() != filteredIt)
		{
			m_filteredAdapters.erase(filteredIt);

			//
			// "Delete" will be reported whenever "Add" has been reported first.
			// m_filter() should not be called.
			//
			m_updateSink(
				m_filteredAdapters,
				&iface,
				UpdateType::Delete
			);
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
