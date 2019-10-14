#pragma once

#include <libcommon/logging/ilogsink.h>
#include <map>
#include <mutex>
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
		std::shared_ptr<NetworkAdapterMonitor::WinNotifier> winNotifier,
		std::function<void(std::map<ULONG64, NetworkAdapterMonitor::AdapterElement> &adaptersOut)> initAdapters
	);
	OfflineMonitor(std::shared_ptr<common::logging::ILogSink> logSink, Notifier notifier, std::shared_ptr<NetworkAdapterMonitor::WinNotifier> winNotifier);
	OfflineMonitor(std::shared_ptr<common::logging::ILogSink> logSink, Notifier notifier);

private:

	std::shared_ptr<common::logging::ILogSink> m_logSink;
	Notifier m_notifier;

	bool m_connected;
	NetworkAdapterMonitor m_netInterfaces;

	void LogOfflineState();

protected:

	NetworkAdapterMonitor& getAdapter()
	{
		return m_netInterfaces;
	}

	void callback(const std::vector<MIB_IF_ROW2> &adapters, const MIB_IF_ROW2 *adapter, NetworkAdapterMonitor::UpdateType type);
};
