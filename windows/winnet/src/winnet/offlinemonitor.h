#pragma once

#include <libcommon/logging/ilogsink.h>
#include <mutex>
#include <optional>
#include "networkadaptermonitor.h"

class OfflineMonitor
{
public:

	//
	// Connectivity changed.
	// true = connected, false = disconnected.
	//
	using Notifier = std::function<void(bool)>;

	OfflineMonitor(
		std::shared_ptr<common::logging::ILogSink> logSink,
		Notifier notifier,
		std::shared_ptr<NetworkAdapterMonitor::IDataProvider> dataProvider
	);

	OfflineMonitor(std::shared_ptr<common::logging::ILogSink> logSink, Notifier notifier);

private:

	std::shared_ptr<common::logging::ILogSink> m_logSink;
	Notifier m_notifier;

	std::optional<bool> m_connected;

	NetworkAdapterMonitor m_netAdapterMonitor;

	void callback(const std::vector<MIB_IF_ROW2> &adapters, const MIB_IF_ROW2 *adapter, NetworkAdapterMonitor::UpdateType type);
};
