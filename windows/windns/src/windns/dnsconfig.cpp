#include "stdafx.h"
#include "dnsconfig.h"
#include "windns/comhelpers.h"

DnsConfig::DnsConfig(CComPtr<IWbemClassObject> instance)
{
	//
	// V_xxx macros seem to require an l-value so access the correct field directly instead.
	//

	m_configIndex = ComGetPropertyAlways(instance, L"Index").ulVal;

	m_interfaceIndex = ComGetPropertyAlways(instance, L"InterfaceIndex").ulVal;
	m_interfaceGuid = ComConvertString(ComGetPropertyAlways(instance, L"SettingID").bstrVal);

	m_servers = nchelpers::GetDnsServers(instance);
}
