#include "stdafx.h"
#include "netconfighelpers.h"
#include "comhelpers.h"
#include "wmi/methodcall.h"

namespace nchelpers
{

std::wstring GetConfigId(CComPtr<IWbemClassObject> instance)
{
	return ComConvertString(V_BSTR(&ComGetPropertyAlways(instance, L"SettingID")));
}

OptionalStringList GetDnsServers(CComPtr<IWbemClassObject> instance)
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

void SetDnsServers(wmi::IConnection &connection, CComPtr<IWbemClassObject> instance,
	const std::vector<std::wstring> *servers)
{
	wmi::MethodCall methodCall;

	if (nullptr == servers)
	{
		methodCall.addNullArgument(L"DNSServerSearchOrder", VT_ARRAY | VT_BSTR);
	}
	else
	{
		auto comServers = ComConvertIntoStringArray(*servers);
		methodCall.addArgument(L"DNSServerSearchOrder", ComPackageStringArray(comServers));
	}

	auto status = methodCall.invoke(connection, instance, L"SetDNSServerSearchOrder");

	//
	// TODO check status, (type? expected value?)
	//

	return;
}

}
