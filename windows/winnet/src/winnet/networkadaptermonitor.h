#pragma once

#include <libcommon/logging/ilogsink.h>
#include <libcommon/error.h>
#include <map>
#include <winsock2.h>
#include <ws2ipdef.h>
#include <iphlpapi.h>
#include <windows.h>
#include <functional>
#include <mutex>
#include <vector>


class WinNotifier
{
public:
	virtual ~WinNotifier() = 0
	{
	}
};


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

	//
	// An event may apply to a specific adapter, or it may apply to all adapters.
	// In the latter case, 'adapter' will be set to nullptr.
	//
	using UpdateSinkType = std::function<void(const std::vector<MIB_IF_ROW2> &adapters, const MIB_IF_ROW2 *adapter, UpdateType updateType)>;

	NetworkAdapterMonitor(
		std::shared_ptr<common::logging::ILogSink> logSink
		, UpdateSinkType updateSink
		, FilterType filter
	);
	virtual ~NetworkAdapterMonitor() = default;

	NetworkAdapterMonitor(const NetworkAdapterMonitor &) = delete;
	NetworkAdapterMonitor& operator=(const NetworkAdapterMonitor &) = delete;
	NetworkAdapterMonitor(NetworkAdapterMonitor &&) = delete;
	NetworkAdapterMonitor& operator=(NetworkAdapterMonitor &&) = delete;

	std::shared_ptr<WinNotifier> m_winNotifier;

	const std::vector<MIB_IF_ROW2>& getAdapters() {
		return m_filteredAdapters;
	}

private:

	std::vector<MIB_IF_ROW2>::iterator findFilteredAdapter(const MIB_IF_ROW2 &adapter);

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
	static void __stdcall Callback(void *context, MIB_IPINTERFACE_ROW *hint, MIB_NOTIFICATION_TYPE updateType);

protected:

	NetworkAdapterMonitor() = default;

	std::shared_ptr<common::logging::ILogSink> m_logSink;
	UpdateSinkType m_updateSink;
	FilterType m_filter;

	class DefaultWinNotifier : public WinNotifier
	{
		HANDLE m_notificationHandle;

	public:

		DefaultWinNotifier(const NetworkAdapterMonitor &nam);
		virtual ~DefaultWinNotifier();

		DefaultWinNotifier(const DefaultWinNotifier&) = delete;
		DefaultWinNotifier(DefaultWinNotifier&&) = delete;
		DefaultWinNotifier& operator=(const DefaultWinNotifier&) = delete;
		DefaultWinNotifier& operator=(const DefaultWinNotifier&&) = delete;
	};

	virtual void callback(const MIB_IPINTERFACE_ROW *hint, MIB_NOTIFICATION_TYPE updateType);

	virtual void getIfEntry(MIB_IF_ROW2 &rowOut, NET_LUID luid);
};
