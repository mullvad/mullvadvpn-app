#include "stdafx.h"
#include "netmonitor.h"
#include <libcommon/error.h>
#include <libcommon/memory.h>
#include <libcommon/synchronization.h>

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

} // anonyomus namespace

NetMonitor::NetMonitor(NetMonitor::Notifier notifier, bool &currentConnectivity)
	: m_connected(false)
	, m_notifier(notifier)
	, m_notificationHandle(nullptr)
{
	m_cache = CreateCache();
	updateConnectivity();

	currentConnectivity = m_connected;

	const auto status = NotifyIpInterfaceChange(AF_UNSPEC, Callback, this, FALSE, &m_notificationHandle);

	THROW_UNLESS(NO_ERROR, status, "Register interface change notification");
}

NetMonitor::~NetMonitor()
{
	CancelMibChangeNotify2(m_notificationHandle);
}

// static
bool NetMonitor::CheckConnectivity()
{
	return CheckConnectivity(CreateCache());
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
	auto thiz = reinterpret_cast<NetMonitor *>(context);

	common::sync::ScopeLock<> processingLock(thiz->m_processingMutex);

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

			thiz->AddCacheEntry(thiz->m_cache, iface);

			break;
		}
		case MibDeleteInstance:
		{
			const auto cacheEntry = thiz->m_cache.find(hint->InterfaceLuid.Value);

			if (thiz->m_cache.end() != cacheEntry)
			{
				cacheEntry->second.connected = false;
			}

			break;
		}
		case MibParameterNotification:
		{
			auto cacheEntry = thiz->m_cache.find(hint->InterfaceLuid.Value);

			if (thiz->m_cache.end() == cacheEntry)
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

				thiz->AddCacheEntry(thiz->m_cache, iface);
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

	const auto previousConnectivity = thiz->m_connected;

	thiz->updateConnectivity();

	if (previousConnectivity != thiz->m_connected)
	{
		thiz->m_notifier(thiz->m_connected);
	}
}
