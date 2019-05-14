#pragma once

#include <map>
#include <string>
#include <cstdint>
#include <mutex>
#include <functional>
#include <winsock2.h>
#include <ws2ipdef.h>
#include <iphlpapi.h>
#include <windows.h>

class NetMonitor
{
public:

	//
	// Connectivity changed.
	// true = connected, false = disconnected.
	//
	using Notifier = std::function<void(bool)>;

	NetMonitor(Notifier notifier, bool &currentConnectivity);
	~NetMonitor();

	static bool CheckConnectivity();

private:

	struct CacheEntry
	{
		// Unique interface identifier.
		uint64_t luid;

		// Whether this is a physical adapter or not.
		bool valid;

		// Last known state.
		bool connected;
	};

	using Cache = std::map<uint64_t, CacheEntry>;

	std::mutex m_processingMutex;
	Cache m_cache;
	bool m_connected;
	Notifier m_notifier;

	HANDLE m_notificationHandle;

	static Cache CreateCache();
	static void AddCacheEntry(Cache &cache, const MIB_IF_ROW2 &iface);
	static bool CheckConnectivity(const Cache &cache);

	void updateConnectivity();

	static void __stdcall Callback(void *context, MIB_IPINTERFACE_ROW *hint, MIB_NOTIFICATION_TYPE updateType);
};
