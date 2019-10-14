#pragma once

#include <WinSock2.h>

#include "libcommon/logging/logsink.h"
#include "../../winnet/networkadaptermonitor.h"

using FilterType = NetworkAdapterMonitor::FilterType;
using UpdateSinkType = NetworkAdapterMonitor::UpdateSinkType;
using UpdateType = NetworkAdapterMonitor::UpdateType;


class NetworkAdapterMonitorTester;

class TestWinNotifier : public NetworkAdapterMonitor::WinNotifier
{
	std::shared_ptr<common::logging::ILogSink> m_logSink;
	AdapterUpdate m_callback;

public:

	TestWinNotifier() {}

	void sendEvent(const MIB_IPINTERFACE_ROW *hint, MIB_NOTIFICATION_TYPE updateType) const;
	void attach(std::shared_ptr<common::logging::ILogSink> logSink, AdapterUpdate callback) override;
	void detach() override;
};

class NetworkAdapterMonitorTester : public NetworkAdapterMonitor
{
public:

	NetworkAdapterMonitorTester(
		std::shared_ptr<common::logging::ILogSink> logSink
		, FilterType filter
		, std::shared_ptr<WinNotifier> notifier
	) : NetworkAdapterMonitor(logSink, [](const std::vector<MIB_IF_ROW2> &adapters, const MIB_IF_ROW2 *adapter, UpdateType updateType) -> void {}, filter, notifier, [](std::map<ULONG64, AdapterElement> &adaptersOut) {})
	{
	}
	virtual ~NetworkAdapterMonitorTester() = default;

	virtual void callback(const MIB_IPINTERFACE_ROW *hint, MIB_NOTIFICATION_TYPE updateType) override
	{
		NetworkAdapterMonitor::callback(hint, updateType);
	}

private:

	void getIfEntry(MIB_IF_ROW2 &rowOut, NET_LUID luid) override;
};
