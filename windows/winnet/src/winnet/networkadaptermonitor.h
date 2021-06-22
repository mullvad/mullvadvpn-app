#pragma once

#include <libcommon/logging/ilogsink.h>
#include <libcommon/error.h>
#include <map>
#include <winsock2.h>
#include <ws2ipdef.h>
#include <iphlpapi.h>
#include <windows.h>
#include <functional>
#include <vector>
#include <mutex>
#include <optional>

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

	struct IDataProvider;
	class SystemDataProvider;

	NetworkAdapterMonitor(
		std::shared_ptr<common::logging::ILogSink> logSink
		, UpdateSinkType updateSink
		, FilterType filter
		, std::shared_ptr<IDataProvider> dataProvider
	);
	NetworkAdapterMonitor(
		std::shared_ptr<common::logging::ILogSink> logSink
		, UpdateSinkType updateSink
		, FilterType filter
	);
	~NetworkAdapterMonitor();

	NetworkAdapterMonitor(const NetworkAdapterMonitor &) = delete;
	NetworkAdapterMonitor& operator=(const NetworkAdapterMonitor &) = delete;
	NetworkAdapterMonitor(NetworkAdapterMonitor &&) = delete;
	NetworkAdapterMonitor& operator=(NetworkAdapterMonitor &&) = delete;

private:

	std::shared_ptr<common::logging::ILogSink> m_logSink;
	UpdateSinkType m_updateSink;
	FilterType m_filter;

	std::shared_ptr<IDataProvider> m_dataProvider;

	std::mutex m_callbackLock;

	std::optional<MIB_IF_ROW2> getAdapter(NET_LUID luid) const;

	bool hasIPv4Interface(NET_LUID luid) const;
	bool hasIPv6Interface(NET_LUID luid) const;

	std::map<ULONG64, MIB_IF_ROW2> m_adapters;
	std::vector<MIB_IF_ROW2> m_filteredAdapters;

	std::vector<MIB_IF_ROW2>::iterator findFilteredAdapter(const NET_LUID adapter);

	HANDLE m_notificationHandle;
	static void __stdcall Callback(void *context, MIB_IPINTERFACE_ROW *hint, MIB_NOTIFICATION_TYPE updateType);
	virtual void callback(const MIB_IPINTERFACE_ROW *hint, MIB_NOTIFICATION_TYPE updateType);
};


struct NetworkAdapterMonitor::IDataProvider
{
	virtual ~IDataProvider() = 0
	{
	}

	virtual DWORD notifyIpInterfaceChange(
		ADDRESS_FAMILY Family,
		PIPINTERFACE_CHANGE_CALLBACK Callback,
		PVOID CallerContext,
		BOOLEAN InitialNotification,
		HANDLE *NotificationHandle
	) = 0;
	virtual DWORD cancelMibChangeNotify2(HANDLE NotificationHandle) = 0;

	virtual DWORD getIfTable2(PMIB_IF_TABLE2 *Table) = 0;
	virtual void freeMibTable(PVOID Memory) = 0;
	
	virtual DWORD getIfEntry2(PMIB_IF_ROW2 Row) = 0;
	virtual DWORD getIpInterfaceEntry(PMIB_IPINTERFACE_ROW Row) = 0;
};

class NetworkAdapterMonitor::SystemDataProvider : public IDataProvider
{
public:

	SystemDataProvider() = default;
	virtual ~SystemDataProvider() = default;

	SystemDataProvider(const SystemDataProvider&) = delete;
	SystemDataProvider(SystemDataProvider&&) = delete;
	SystemDataProvider& operator=(const SystemDataProvider&) = delete;
	SystemDataProvider& operator=(const SystemDataProvider&&) = delete;

	DWORD notifyIpInterfaceChange(
		ADDRESS_FAMILY Family,
		PIPINTERFACE_CHANGE_CALLBACK Callback,
		PVOID CallerContext,
		BOOLEAN InitialNotification,
		HANDLE *NotificationHandle
	) override;
	DWORD cancelMibChangeNotify2(HANDLE NotificationHandle) override;

	DWORD getIfTable2(PMIB_IF_TABLE2 *Table) override;
	void freeMibTable(PVOID Memory) override;
	
	DWORD getIfEntry2(PMIB_IF_ROW2 Row) override;
	DWORD getIpInterfaceEntry(PMIB_IPINTERFACE_ROW Row) override;
};
