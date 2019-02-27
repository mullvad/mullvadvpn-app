#pragma once

#include "ifirewallrule.h"
#include "libwfp/ipaddress.h"
#include <string>

namespace rules
{

class RestrictDns : public IFirewallRule
{
public:

	RestrictDns(const std::wstring &tunnelInterfaceAlias, const wfp::IpAddress v4DnsHost, wfp::IpAddress *v6DnsHost);
	
	bool apply(IObjectInstaller &objectInstaller) override;

private:

	const std::wstring m_tunnelInterfaceAlias;
	const wfp::IpAddress m_v4DnsHost;
	const wfp::IpAddress *m_v6DnsHost;

};

}
