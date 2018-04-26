#pragma once

#include <string>
#include <memory>
#include <vector>
#include <atlbase.h>
#include <wbemidl.h>

namespace dnshelpers
{

// instance = Win32_NetworkAdapterConfiguration
std::wstring GetId(CComPtr<IWbemClassObject> instance);

using OptionalStringList = std::unique_ptr<std::vector<std::wstring> >;

// instance = Win32_NetworkAdapterConfiguration
OptionalStringList GetServers(CComPtr<IWbemClassObject> instance);

}