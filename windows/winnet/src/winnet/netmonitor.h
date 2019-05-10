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

private:

	std::mutex m_processingMutex;

	Notifier m_notifier;

	struct CacheEntry
	{
		// Unique interface identifier.
		uint64_t luid;

		// Whether this is a physical adapter or not.
		bool valid;

		// Last known state.
		bool connected;
	};

	std::map<uint64_t, CacheEntry> m_cache;

	bool m_connected;

	HANDLE m_notificationHandle;

	void createCache();
	void addCacheEntry(const MIB_IF_ROW2 &iface);
	void updateConnectivity();

	static void __stdcall callback(void *context, MIB_IPINTERFACE_ROW *hint, MIB_NOTIFICATION_TYPE updateType);
};
