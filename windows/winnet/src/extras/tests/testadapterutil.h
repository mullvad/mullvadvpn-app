#pragma once

#include <functional>
#include <winSock2.h>
#include <winnet/networkadaptermonitor.h>

using FilterType = NetworkAdapterMonitor::FilterType;
using UpdateSinkType = NetworkAdapterMonitor::UpdateSinkType;
using UpdateType = NetworkAdapterMonitor::UpdateType;


class MibIfTable
{
	std::vector<MIB_IF_ROW2> m_table;
	
public:

	void add(const MIB_IF_ROW2 &row);
	void remove(const MIB_IF_ROW2 &row);

	const std::vector<MIB_IF_ROW2>& entries() const
	{
		return m_table;
	}
};

class TestDataProvider : public NetworkAdapterMonitor::IDataProvider
{

	PIPINTERFACE_CHANGE_CALLBACK m_callback;
	void *m_context;

	MibIfTable m_adapterTable;
	std::vector<MIB_IPINTERFACE_ROW> m_ipInterfaces;

public:

	TestDataProvider() : m_callback(nullptr), m_context(nullptr)
	{
	}

	TestDataProvider(const TestDataProvider&) = delete;
	TestDataProvider(TestDataProvider&&) = delete;
	TestDataProvider& operator=(const TestDataProvider&) = delete;
	TestDataProvider& operator=(const TestDataProvider&&) = delete;

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

	//
	// Test utilities
	//

	void addAdapter(const MIB_IF_ROW2 &adapter);
	void addIpInterface(const MIB_IF_ROW2 &adapter, const MIB_IPINTERFACE_ROW &iface);

	void removeAdapter(const MIB_IF_ROW2 &adapter);
	void removeIpInterface(const MIB_IPINTERFACE_ROW &iface);
	
	void sendEvent(MIB_IPINTERFACE_ROW *hint, MIB_NOTIFICATION_TYPE updateType);
};
