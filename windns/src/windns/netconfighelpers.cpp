#include "stdafx.h"
#include "dnshelpers.h"
#include "comhelpers.h"

namespace dnshelpers
{

std::wstring GetId(CComPtr<IWbemClassObject> instance)
{
	return ComConvertString(V_BSTR(&ComGetPropertyAlways(instance, L"SettingID")));
}

OptionalStringList GetServers(CComPtr<IWbemClassObject> instance)
{
	OptionalStringList result;

	auto servers = ComGetProperty(instance, L"DNSServerSearchOrder");

	if (VT_EMPTY == V_VT(&servers) || VT_NULL == V_VT(&servers))
	{
		return result;
	}

	result = std::make_unique<std::vector<std::wstring> >(
		ComConvertStringArray(V_ARRAY(&servers)));

	return result;
}

}
