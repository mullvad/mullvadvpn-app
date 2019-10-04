#pragma once

#include <libcommon/logging/ilogsink.h>
#include <map>
#include <mutex>
#include "networkadaptermonitor.h"

class NetMonitor
{
public:

	//
	// Connectivity changed.
	// true = connected, false = disconnected.
	//
	using Notifier = std::function<void(bool)>;

	NetMonitor(std::shared_ptr<common::logging::ILogSink> logSink, Notifier notifier, bool &currentConnectivity);

private:

	std::shared_ptr<common::logging::ILogSink> m_logSink;
	Notifier m_notifier;

	bool m_connected;
	NetworkAdapterMonitor m_netInterfaces;
	void UpdateConnectivity();

	void callback(const MIB_IF_ROW2 &adapter, NetworkAdapterMonitor::UpdateType type);

	void LogOfflineState();
};
