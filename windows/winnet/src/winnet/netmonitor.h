#pragma once

#include <libcommon/logging/ilogsink.h>
#include <map>
#include <mutex>
#include "NetworkInterfaceStatuses.h"

class NetMonitor
{
public:

	//
	// Connectivity changed.
	// true = connected, false = disconnected.
	//
	using Notifier = std::function<void(bool)>;

	NetMonitor(std::shared_ptr<common::logging::ILogSink> logSink, Notifier notifier, bool &currentConnectivity);
	~NetMonitor();

private:

	std::shared_ptr<common::logging::ILogSink> m_logSink;
	Notifier m_notifier;

	std::mutex m_processingMutex;

	HANDLE m_notificationHandle;

	bool m_connected;
	NetworkInterfaceStatuses m_netInterfaces;
	void UpdateConnectivity();

	static void __stdcall Callback(void *context, MIB_IPINTERFACE_ROW *hint, MIB_NOTIFICATION_TYPE updateType);
	void callback(MIB_IPINTERFACE_ROW *hint, MIB_NOTIFICATION_TYPE updateType);

	void LogOfflineState();
};
