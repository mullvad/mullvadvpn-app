#pragma once

#include "interfacesnap.h"
#include "interfacemonitor.h"
#include "types.h"
#include "inameserversource.h"
#include "irecoverysink.h"
#include "ilogsink.h"
#include <libcommon/registry/registry.h>
#include <string>
#include <vector>
#include <memory>
#include <windows.h>

//
// DnsAgent:
// Monitor interfaces and enforce name server settings.
//
class DnsAgent
{
public:

	DnsAgent(Protocol protocol, INameServerSource *nameServerSource, IRecoverySink *recoverySink, ILogSink *logSink);
	~DnsAgent();

private:

	Protocol m_protocol;
	INameServerSource *m_nameServerSource;
	IRecoverySink *m_recoverySink;
	ILogSink *m_logSink;

	//
	// InterfaceData:
	// Tracking entry for a network interface.
	//
	struct InterfaceData
	{
		InterfaceData(const std::wstring &interfaceGuid_, const InterfaceSnap &snap_, std::unique_ptr<InterfaceMonitor> &&monitor_)
			: interfaceGuid(interfaceGuid_), preservedSettings(snap_), monitor(std::move(monitor_))
		{
		}

		std::wstring interfaceGuid;
		InterfaceSnap preservedSettings;
		std::unique_ptr<InterfaceMonitor> monitor;
	};

	std::vector<InterfaceData> m_trackedInterfaces;

	std::unique_ptr<common::registry::RegistryMonitor> m_rootMonitor;

	HANDLE m_serverSourceEvent;
	HANDLE m_thread;
	HANDLE m_shutdownEvent;

	void constructNameServerUpdateEvent();
	void constructRootMonitor();
	void constructThread();

	static unsigned __stdcall ThreadEntry(void *);
	void thread();

	void processServerSourceEvent();

	enum class ProcessingResult
	{
		TrackingUpdated,
		Nop
	};

	ProcessingResult processRootKeyEvent();
	ProcessingResult processInterfaceEvent(const HANDLE *interfaceEvents, size_t startIndex);

	std::vector<std::wstring> discoverInterfaces();

	void setNameServers(const std::wstring &interfaceGuid, const std::vector<std::wstring> &enforcedServers);

	bool startTrackingInterfaces(const std::vector<std::wstring> &interfaces);
	void stopTrackingInterfaces(const std::vector<std::wstring> &interfaces);

	void updateRecoveryData();
};
