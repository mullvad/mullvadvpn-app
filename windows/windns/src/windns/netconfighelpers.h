#pragma once

#include "wmi/iconnection.h"
#include <string>
#include <memory>
#include <vector>
#include <cstdint>
#include <atlbase.h>
#include <wbemidl.h>

namespace nchelpers
{

using OptionalStringList = std::shared_ptr<std::vector<std::wstring> >;

// instance = Win32_NetworkAdapterConfiguration
OptionalStringList GetDnsServers(CComPtr<IWbemClassObject> instance);

// instance = Win32_NetworkAdapterConfiguration
uint32_t GetInterfaceIndex(CComPtr<IWbemClassObject> instance);

void SetDnsServers(uint32_t interfaceIndex, const std::vector<std::wstring> &servers);

}
