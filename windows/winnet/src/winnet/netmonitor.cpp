#include "stdafx.h"
#include "netmonitor.h"
#include <libcommon/error.h>
#include <libcommon/memory.h>
#include <libcommon/string.h>
#include <sstream>

namespace
{

bool ValidInterfaceType(const MIB_IF_ROW2 &iface)
{
	switch (iface.InterfaceLuid.Info.IfType)
	{
		case IF_TYPE_SOFTWARE_LOOPBACK:
		case IF_TYPE_TUNNEL:
		{
			return false;
		}
	}

	if (FALSE != iface.InterfaceAndOperStatusFlags.FilterInterface
		|| 0 == iface.PhysicalAddressLength
		|| FALSE != iface.InterfaceAndOperStatusFlags.EndPointInterface)
	{
		return false;
	}

	return true;
}

} // anonymous namespace

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
	m_cache = CreateCache();
	updateConnectivity();

	currentConnectivity = m_connected;

	const auto status = NotifyIpInterfaceChange(AF_UNSPEC, Callback, this, FALSE, &m_notificationHandle);

	THROW_UNLESS(NO_ERROR, status, "Register interface change notification");

	if (false == m_connected)
	{
		LogOfflineState(m_logSink);
	}
}

NetMonitor::~NetMonitor()
{
	CancelMibChangeNotify2(m_notificationHandle);
}

// static
bool NetMonitor::CheckConnectivity(std::shared_ptr<common::logging::ILogSink> logSink)
{
	static bool loggedOffline = false;

	const auto connected = CheckConnectivity(CreateCache());

	if (connected)
	{
		loggedOffline = false;
	}
	else if (false == loggedOffline)
	{
		LogOfflineState(logSink);
		loggedOffline = true;
	}

	return connected;
}

// static
NetMonitor::Cache NetMonitor::CreateCache()
{
	MIB_IF_TABLE2 *table;

	const auto status = GetIfTable2(&table);

	THROW_UNLESS(NO_ERROR, status, "Acquire network interface table");

	common::memory::ScopeDestructor sd;

	sd += [table]()
	{
		FreeMibTable(table);
	};

	std::map<uint64_t, CacheEntry> cache;

	for (ULONG i = 0; i < table->NumEntries; ++i)
	{
		AddCacheEntry(cache, table->Table[i]);
	}

	return cache;
}

// static
void NetMonitor::AddCacheEntry(Cache &cache, const MIB_IF_ROW2 &iface)
{
	CacheEntry e;

	if (ValidInterfaceType(iface))
	{
		e.luid = iface.InterfaceLuid.Value;
		e.valid = true;
		e.connected = (MediaConnectStateConnected == iface.MediaConnectState);
	}
	else
	{
		e.luid = iface.InterfaceLuid.Value;
		e.valid = false;
		e.connected = false;
	}

	cache.insert(std::make_pair(e.luid, e));
}

// static
bool NetMonitor::CheckConnectivity(const Cache &cache)
{
	for (const auto cacheEntryIter : cache)
	{
		const auto entry = cacheEntryIter.second;

		if (entry.valid && entry.connected)
		{
			return true;
		}
	}

	return false;
}

void NetMonitor::updateConnectivity()
{
	m_connected = NetMonitor::CheckConnectivity(m_cache);
}

//static
void __stdcall NetMonitor::Callback(void *context, MIB_IPINTERFACE_ROW *hint, MIB_NOTIFICATION_TYPE updateType)
{
	reinterpret_cast<NetMonitor *>(context)->callback(hint, updateType);
}

void NetMonitor::callback(MIB_IPINTERFACE_ROW *hint, MIB_NOTIFICATION_TYPE updateType)
{
	std::scoped_lock<std::mutex> processingLock(m_processingMutex);

	switch (updateType)
	{
		case MibAddInstance:
		{
			MIB_IF_ROW2 iface = { 0 };
			iface.InterfaceLuid = hint->InterfaceLuid;

			if (NO_ERROR != GetIfEntry2(&iface))
			{
				// Failed to query interface.
				return;
			}

			AddCacheEntry(m_cache, iface);

			break;
		}
		case MibDeleteInstance:
		{
			const auto cacheEntry = m_cache.find(hint->InterfaceLuid.Value);

			if (m_cache.end() != cacheEntry)
			{
				cacheEntry->second.connected = false;
			}

			break;
		}
		case MibParameterNotification:
		{
			auto cacheEntry = m_cache.find(hint->InterfaceLuid.Value);

			if (m_cache.end() == cacheEntry)
			{
				//
				// A change occurred on an interface that we're not tracking.
				// Perhaps the MibAddInstance logic failed for some reason.
				//

				MIB_IF_ROW2 iface = { 0 };
				iface.InterfaceLuid = hint->InterfaceLuid;

				if (NO_ERROR != GetIfEntry2(&iface))
				{
					// Failed to query interface.
					return;
				}

				AddCacheEntry(m_cache, iface);
			}
			else
			{
				//
				// Abort processing if this is a known interface that we don't care about.
				//
				if (false == cacheEntry->second.valid)
				{
					return;
				}

				//
				// Update cache.
				//

				MIB_IF_ROW2 iface = { 0 };
				iface.InterfaceLuid = hint->InterfaceLuid;

				const auto status = GetIfEntry2(&iface);

				cacheEntry->second.connected =
					(NO_ERROR == status ? MediaConnectStateConnected == iface.MediaConnectState : false);
			}

			break;
		}
	}

	const auto previousConnectivity = m_connected;

	updateConnectivity();

	if (previousConnectivity != m_connected)
	{
		m_notifier(m_connected);

		if (false == m_connected)
		{
			LogOfflineState(m_logSink);
		}
	}
}

//static
void NetMonitor::LogOfflineState(std::shared_ptr<common::logging::ILogSink> logSink)
{
	//
	// There is a race condition here because logging is not done using the
	// same data set that the online/offline logic processes.
	//
	// Not much of a problem really, this is temporary logging.
	//

	logSink->info("Machine is offline");

	MIB_IF_TABLE2 *table;

	const auto status = GetIfTable2(&table);

	if (NO_ERROR != status)
	{
		logSink->error("Failed to acquire list of network interfaces. Aborting detailed logging");
		return;
	}

	common::memory::ScopeDestructor sd;

	sd += [table]()
	{
		FreeMibTable(table);
	};

	logSink->info("Begin detailed listing of network interfaces");

	for (ULONG i = 0; i < table->NumEntries; ++i)
	{
		const auto &iface = table->Table[i];

		std::stringstream ss;

		ss << "Detailed interface logging" << std::endl;
		ss << "Interface ordinal " << i << std::endl;

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
			ss << "  filtered out to avoid flooding log";
			logSink->info(ss.str().c_str());

			continue;
		}

		{
			const auto s = std::wstring(L"  Alias: ").append(iface.Alias);
			ss << common::string::ToAnsi(s) << std::endl;
		}

		{
			const auto s = std::wstring(L"  \"Description\": ").append(iface.Description);
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

		logSink->info(ss.str().c_str());
	}

	logSink->info("End detailed listing of network interfaces");
}
