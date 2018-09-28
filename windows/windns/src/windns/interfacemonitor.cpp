#include "stdafx.h"
#include "interfacemonitor.h"
#include "registrypaths.h"

using namespace common::registry;

InterfaceMonitor::InterfaceMonitor(Protocol protocol, const std::wstring &interfaceGuid)
	: m_protocol(protocol)
	, m_interfaceGuid(interfaceGuid)
{
	const auto interfacePath = RegistryPaths::InterfaceKey(interfaceGuid, protocol);

	m_monitor = Registry::MonitorKey(HKEY_LOCAL_MACHINE, interfacePath, { RegistryEventFlag::ValueChange });
}

HANDLE InterfaceMonitor::queueSingleEvent()
{
	return m_monitor->queueSingleEvent();
}

const std::wstring &InterfaceMonitor::interfaceGuid() const
{
	return m_interfaceGuid;
}
