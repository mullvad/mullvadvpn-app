#pragma once

#include <libcommon/logging/ilogsink.h>
#include <map>
#include <winsock2.h>
#include <ws2ipdef.h>
#include <iphlpapi.h>
#include <windows.h>
#include <functional>
#include <mutex>
#include <vector>


class NetworkAdapterMonitor
{
public:

	enum class UpdateType
	{
		Add,
		Delete,
		Update
	};

	using FilterType = std::function<bool(const MIB_IF_ROW2 &adapter)>;
	using UpdateSinkType = std::function<void(const MIB_IF_ROW2 &adapter, UpdateType updateType)>;

	NetworkAdapterMonitor(
		std::shared_ptr<common::logging::ILogSink> logSink
		, UpdateSinkType updateSink
		, FilterType filter
	);
	~NetworkAdapterMonitor();

	NetworkAdapterMonitor(const NetworkAdapterMonitor &o) = delete;
	NetworkAdapterMonitor& operator=(const NetworkAdapterMonitor &o) = delete;
	NetworkAdapterMonitor(NetworkAdapterMonitor &&o) = delete;
	NetworkAdapterMonitor& operator=(NetworkAdapterMonitor &&o) = delete;

	const std::vector<MIB_IF_ROW2>& getFilteredAdapters() const;

private:

	std::shared_ptr<common::logging::ILogSink> m_logSink;

	UpdateSinkType m_updateSink;
	FilterType m_filter;

	void addInternal(
		const MIB_IF_ROW2 &newIface,
		bool IPv4,
		bool IPv6
	);
	void addFilteredIfUnique(const MIB_IF_ROW2 &adapter);

	struct AdapterElement
	{
		AdapterElement(
			const MIB_IF_ROW2 &adapter,
			bool ipv4Enabled,
			bool ipv6Enabled
		)
			: adapter(adapter)
			, IPv4(ipv4Enabled)
			, IPv6(ipv6Enabled)
		{
		}

		bool IPv4;
		bool IPv6;
		MIB_IF_ROW2 adapter;
	};

	std::map<ULONG64, AdapterElement> m_adapters;
	std::vector<MIB_IF_ROW2> m_filteredAdapters;

	std::mutex m_processingMutex;
	HANDLE m_notificationHandle;
	void callback(const MIB_IPINTERFACE_ROW *hint, MIB_NOTIFICATION_TYPE updateType);
	static void __stdcall Callback(void *context, MIB_IPINTERFACE_ROW *hint, MIB_NOTIFICATION_TYPE updateType);
};
