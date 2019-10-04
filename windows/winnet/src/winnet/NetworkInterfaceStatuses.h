#pragma once

#include <libcommon/logging/ilogsink.h>
#include <map>
#include <winsock2.h>
#include <ws2ipdef.h>
#include <iphlpapi.h>
#include <windows.h>
#include <functional>
#include <mutex>


class NetworkInterfaceStatuses
{
public:

	NetworkInterfaceStatuses(
		std::shared_ptr<common::logging::ILogSink> logSink,
		std::function<bool(const MIB_IF_ROW2 &adapter)> filter,
		std::function<void(const MIB_IPINTERFACE_ROW *hint, bool connected)> updateSink
	);
	~NetworkInterfaceStatuses();

	NetworkInterfaceStatuses(NetworkInterfaceStatuses &o) = delete;
	NetworkInterfaceStatuses& operator=(const NetworkInterfaceStatuses &o) = delete;

	bool anyConnected() const;
	bool connected(NET_LUID luid) const;

private:

	std::shared_ptr<common::logging::ILogSink> m_logSink;

	// the callback is called whenever an interface is connected or disconnected
	std::function<void(const MIB_IPINTERFACE_ROW *hint, bool connected)> m_updateSink;

	void add(NET_LUID luid);
	void remove(NET_LUID luid);
	void update(NET_LUID luid);

	struct Entry
	{
		// Last known state.
		bool connected;

		// Unique interface identifier.
		uint64_t luid;

		Entry(uint64_t luid, bool connected) :
			luid(luid)
			, connected(connected)
		{
		}

		Entry() :
			connected(false)
			, luid(0)
		{
		}
	};

	std::map<uint64_t, Entry> m_interfaces;

	void addInternal(const MIB_IF_ROW2 &iface);

	std::mutex m_processingMutex;
	HANDLE m_notificationHandle;
	void callback(const MIB_IPINTERFACE_ROW *hint, MIB_NOTIFICATION_TYPE updateType);
	static void __stdcall Callback(void *context, MIB_IPINTERFACE_ROW *hint, MIB_NOTIFICATION_TYPE updateType);
};
