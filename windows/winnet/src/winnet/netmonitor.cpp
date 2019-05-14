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

	if (FALSE == iface.InterfaceAndOperStatusFlags.ConnectorPresent
		|| FALSE != iface.InterfaceAndOperStatusFlags.EndPointInterface)
	{
		return false;
	}

	return true;
}

} // anonyomus namespace

NetMonitor::NetMonitor(NetMonitor::Notifier notifier, bool &currentConnectivity)
	: m_notifier(notifier)
	, m_connected(false)
	, m_notificationHandle(nullptr)
{
	m_cache = createCache();
	updateConnectivity();

	currentConnectivity = m_connected;

	const auto status = NotifyIpInterfaceChange(AF_UNSPEC, callback, this, FALSE, &m_notificationHandle);

	THROW_UNLESS(NO_ERROR, status, "Register interface change notification");
}

bool NetMonitor::checkConnectivity() {
	return NetMonitor::checkConnectivity(NetMonitor::createCache());
}

NetMonitor::~NetMonitor()
{
	CancelMibChangeNotify2(m_notificationHandle);
}

std::map<uint64_t, NetMonitor::CacheEntry> NetMonitor::createCache()
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
		addCacheEntry(table->Table[i], cache);
	}
	return cache;
}

void NetMonitor::addCacheEntry(const MIB_IF_ROW2 &iface, std::map<uint64_t, NetMonitor::CacheEntry> &cache)
{
	CacheEntry e;

	if (false == ValidInterfaceType(iface))
	{
		e.luid = iface.InterfaceLuid.Value;
		e.valid = false;
		e.connected = false;
	}
	else
	{
		e.luid = iface.InterfaceLuid.Value;
		e.valid = true;
		e.connected = (MediaConnectStateConnected == iface.MediaConnectState);
	}

	auto pair = std::make_pair(e.luid, e);
	cache.insert(pair);
}


bool NetMonitor::checkConnectivity(const std::map<uint64_t, NetMonitor::CacheEntry> &cache) {
	for (const auto cacheEntryIter : cache) {
		const auto entry = cacheEntryIter.second;
		if (entry.valid && entry.connected) {
			return true;
		}
	}

	return false;
}

void NetMonitor::updateConnectivity()
{
	m_connected = NetMonitor::checkConnectivity(m_cache);
}

//static
void __stdcall NetMonitor::callback(void *context, MIB_IPINTERFACE_ROW *hint, MIB_NOTIFICATION_TYPE updateType)
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

			thiz->addCacheEntry(iface, thiz->m_cache);

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

				thiz->addCacheEntry(iface, thiz->m_cache);
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
