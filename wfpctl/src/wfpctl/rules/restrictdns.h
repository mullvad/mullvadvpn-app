#pragma once

#include "ifirewallrule.h"
#include "libwfp/ipaddress.h"
#include <string>

namespace rules
{

class RestrictDns : public IFirewallRule
{
public:

	RestrictDns(const std::wstring &tunnelInterfaceAlias, const wfp::IpAddress &dns);
	
	bool apply(IObjectInstaller &objectInstaller) override;

private:

	const std::wstring m_tunnelInterfaceAlias;
	const wfp::IpAddress m_dns;
};

}
