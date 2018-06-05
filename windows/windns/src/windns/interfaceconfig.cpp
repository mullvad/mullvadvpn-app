#include "stdafx.h"
#include "interfaceconfig.h"
#include "windns/comhelpers.h"

InterfaceConfig::InterfaceConfig(CComPtr<IWbemClassObject> instance)
{
	//
	// V_xxx macros seem to require an l-value so access the correct field directly instead.
	//

	m_configIndex = ComGetPropertyAlways(instance, L"Index").ulVal;

	m_dhcp = ComGetPropertyAlways(instance, L"DHCPEnabled").boolVal;

	m_interfaceIndex = ComGetPropertyAlways(instance, L"InterfaceIndex").ulVal;
	m_interfaceGuid = ComConvertString(ComGetPropertyAlways(instance, L"SettingID").bstrVal);

	m_servers = nchelpers::GetDnsServers(instance);
}
