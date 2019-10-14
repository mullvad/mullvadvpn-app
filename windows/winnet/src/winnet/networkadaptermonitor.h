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

	class WinNotifier
	{
	public:
		using AdapterUpdate = std::function<void(const MIB_IPINTERFACE_ROW *hint, MIB_NOTIFICATION_TYPE updateType)>;

		virtual ~WinNotifier() = 0
		{
		}

		virtual void attach(std::shared_ptr<common::logging::ILogSink> logSink, AdapterUpdate callback) = 0;
		virtual void detach() = 0;
	};

	class DefaultWinNotifier : public WinNotifier
	{
		HANDLE m_notificationHandle;
		bool m_attached;
		std::shared_ptr<common::logging::ILogSink> m_logSink;
		AdapterUpdate m_callback;

		static void __stdcall Callback(void *context, MIB_IPINTERFACE_ROW *hint, MIB_NOTIFICATION_TYPE updateType);

	public:

		DefaultWinNotifier();
		virtual ~DefaultWinNotifier();

		DefaultWinNotifier(const DefaultWinNotifier&) = delete;
		DefaultWinNotifier(DefaultWinNotifier&&) = delete;
		DefaultWinNotifier& operator=(const DefaultWinNotifier&) = delete;
		DefaultWinNotifier& operator=(const DefaultWinNotifier&&) = delete;

		void attach(std::shared_ptr<common::logging::ILogSink> logSink, AdapterUpdate callback) override;
		void detach() override;
	};

	NetworkAdapterMonitor(
		std::shared_ptr<common::logging::ILogSink> logSink
		, UpdateSinkType updateSink
		, FilterType filter
		, std::shared_ptr<WinNotifier> notifier
		, std::function<void(std::map<ULONG64, MIB_IF_ROW2> &adaptersOut)> initAdapters
	);
	NetworkAdapterMonitor(
		std::shared_ptr<common::logging::ILogSink> logSink
		, UpdateSinkType updateSink
		, FilterType filter
		, std::shared_ptr<WinNotifier> notifier
	);
	NetworkAdapterMonitor(
		std::shared_ptr<common::logging::ILogSink> logSink
		, UpdateSinkType updateSink
		, FilterType filter
	) : NetworkAdapterMonitor(logSink, updateSink, filter, std::make_shared<DefaultWinNotifier>())
	{
	}
	virtual ~NetworkAdapterMonitor();

	NetworkAdapterMonitor(const NetworkAdapterMonitor &) = delete;
	NetworkAdapterMonitor& operator=(const NetworkAdapterMonitor &) = delete;
	NetworkAdapterMonitor(NetworkAdapterMonitor &&) = delete;
	NetworkAdapterMonitor& operator=(NetworkAdapterMonitor &&) = delete;

	const std::vector<MIB_IF_ROW2>& getAdapters() const {
		return m_filteredAdapters;
	}

private:

	std::shared_ptr<WinNotifier> m_winNotifier;

	std::vector<MIB_IF_ROW2>::iterator findFilteredAdapter(const MIB_IF_ROW2 &adapter);

	std::map<ULONG64, MIB_IF_ROW2> m_adapters;
	std::vector<MIB_IF_ROW2> m_filteredAdapters;

	std::mutex m_processingMutex;
	
protected:

	std::shared_ptr<common::logging::ILogSink> m_logSink;
	UpdateSinkType m_updateSink;
	FilterType m_filter;

	NetworkAdapterMonitor() = default;

	virtual void callback(const MIB_IPINTERFACE_ROW *hint, MIB_NOTIFICATION_TYPE updateType);

	virtual void getIfEntry(MIB_IF_ROW2 &rowOut, NET_LUID luid);
};
