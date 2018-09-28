#pragma once

#include "types.h"
#include <libcommon/registry/registry.h>
#include <string>
#include <memory>
#include <windows.h>

class InterfaceMonitor
{
public:

	explicit InterfaceMonitor(Protocol protocol, const std::wstring &interfaceGuid);

	//
	// The event becomes signalled if:
	// 1. A value change occurs.
	// 2. The monitored interface's registry key is deleted.
	//
	HANDLE queueSingleEvent();

	const std::wstring &interfaceGuid() const;

private:

	Protocol m_protocol;
	std::wstring m_interfaceGuid;

	std::unique_ptr<common::registry::RegistryMonitor> m_monitor;
};
