#include "stdafx.h"
#include "netconfighelpers.h"
#include "comhelpers.h"
#include "wmi/methodcall.h"
#include <cstdint>
#include <sstream>
#include <stdexcept>

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

	const uint32_t STATUS_SUCCESS_NO_REBOOT_REQUIRED = 0;

	if (STATUS_SUCCESS_NO_REBOOT_REQUIRED == V_UI4(&status))
	{
		return;
	}

	std::string msg("Unable to update adapter configuration with new DNS servers");

	try
	{
		auto configIndex = ComGetPropertyAlways(instance, L"Index");
		auto interfaceIndex = ComGetPropertyAlways(instance, L"InterfaceIndex");

		std::stringstream ss;

		ss << "Unable to update adapter with interfaceIndex = " << V_UI4(&interfaceIndex) \
			<< ", configuration index = " << V_UI4(&configIndex) \
			<< " with new DNS servers. Error: " << V_UI4(&status);

		msg = ss.str();
	}
	catch (...)
	{
	}

	throw std::runtime_error(msg);
}

}
