#pragma once

#include "types.h"
#include "interfaceconfig.h"
#include <string>
#include <vector>
#include <cstdint>
#include <atlbase.h>
#include <wbemidl.h>

namespace nchelpers
{

// instance = Win32_NetworkAdapterConfiguration
OptionalStringList GetDnsServers(CComPtr<IWbemClassObject> instance);

// instance = Win32_NetworkAdapterConfiguration
uint32_t GetInterfaceIndex(CComPtr<IWbemClassObject> instance);

void SetDnsServers(uint32_t interfaceIndex, const std::vector<std::wstring> &servers);

void RevertDnsServers(const InterfaceConfig &config, uint32_t timeout = 0);

}
