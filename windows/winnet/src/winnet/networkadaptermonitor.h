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

	struct AdapterElement
	{
		AdapterElement() :
			refcount(1)
		{
		}

		MIB_IF_ROW2 adapter;

	private:

		size_t refcount;

		friend class NetworkAdapterMonitor;
	};

	using Filter = std::function<bool(const MIB_IF_ROW2 &adapter)>;
	using UpdateSink = std::function<void(const MIB_IF_ROW2 &adapter, UpdateType updateType)>;

	NetworkAdapterMonitor(
		std::shared_ptr<common::logging::ILogSink> logSink
		, UpdateSink updateSink
		, Filter filter
	);
	~NetworkAdapterMonitor();

	NetworkAdapterMonitor(const NetworkAdapterMonitor &o) = delete;
	NetworkAdapterMonitor& operator=(const NetworkAdapterMonitor &o) = delete;
	NetworkAdapterMonitor(NetworkAdapterMonitor &&o) = delete;
	NetworkAdapterMonitor& operator=(NetworkAdapterMonitor &&o) = delete;

	const std::map<ULONG64, AdapterElement>& getAdapters() const;

private:

	std::shared_ptr<common::logging::ILogSink> m_logSink;

	std::function<void(const MIB_IF_ROW2 &adapter, UpdateType updateType)> m_updateSink;
	std::function<bool(const MIB_IF_ROW2 &adapter)> m_filter;

	void addInternal(const MIB_IF_ROW2 &iface);

	void add(NET_LUID luid);
	void remove(NET_LUID luid);
	void update(NET_LUID luid);

	std::map<ULONG64, AdapterElement> m_adapters;

	std::mutex m_processingMutex;
	HANDLE m_notificationHandle;
	void callback(const MIB_IPINTERFACE_ROW *hint, MIB_NOTIFICATION_TYPE updateType);
	static void __stdcall Callback(void *context, MIB_IPINTERFACE_ROW *hint, MIB_NOTIFICATION_TYPE updateType);
};
