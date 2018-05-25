#pragma once

#include "wmi/iconnection.h"
#include <string>
#include <memory>
#include <vector>
#include <atlbase.h>
#include <wbemidl.h>

namespace nchelpers
{

// instance = Win32_NetworkAdapterConfiguration
std::wstring GetConfigId(CComPtr<IWbemClassObject> instance);

using OptionalStringList = std::unique_ptr<std::vector<std::wstring> >;

// instance = Win32_NetworkAdapterConfiguration
OptionalStringList GetDnsServers(CComPtr<IWbemClassObject> instance);

// instance = Win32_NetworkAdapterConfiguration
void SetDnsServers(wmi::IConnection &connection, CComPtr<IWbemClassObject> instance,
	const std::vector<std::wstring> *servers);

}
