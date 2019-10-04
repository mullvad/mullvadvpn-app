#pragma once

#include <libcommon/logging/ilogsink.h>
#include <map>
#include <winsock2.h>
#include <ws2ipdef.h>
#include <iphlpapi.h>
#include <windows.h>
#include <functional>
#include <mutex>
#include <set>


class NetworkAdapterMonitor
{
public:

	enum class UpdateType
	{
		Add,
		Delete,
		Update
	};

	NetworkAdapterMonitor(
		std::shared_ptr<common::logging::ILogSink> logSink,
		std::function<void(const MIB_IF_ROW2 &adapter, UpdateType type)> updateSink,
		std::function<bool(const MIB_IF_ROW2 &adapter)> filter
	);
	NetworkAdapterMonitor(
		std::shared_ptr<common::logging::ILogSink> logSink,
		std::function<void(const MIB_IF_ROW2 &adapter, UpdateType updateType)> updateSink
	);
	~NetworkAdapterMonitor();

	NetworkAdapterMonitor(NetworkAdapterMonitor &o) = delete;
	NetworkAdapterMonitor& operator=(const NetworkAdapterMonitor &o) = delete;

	size_t numAdapters() const;

private:

	std::shared_ptr<common::logging::ILogSink> m_logSink;

	std::function<void(const MIB_IF_ROW2 &adapter, UpdateType updateType)> m_updateSink;
	std::function<bool(const MIB_IF_ROW2 &adapter)> m_filter;

	void addInternal(const MIB_IF_ROW2 &iface);

	void add(NET_LUID luid);
	void remove(NET_LUID luid);
	void update(NET_LUID luid);

	std::map<ULONG64, MIB_IF_ROW2> m_adapters;

	std::mutex m_processingMutex;
	HANDLE m_notificationHandle;
	void callback(const MIB_IPINTERFACE_ROW *hint, MIB_NOTIFICATION_TYPE updateType);
	static void __stdcall Callback(void *context, MIB_IPINTERFACE_ROW *hint, MIB_NOTIFICATION_TYPE updateType);
};
