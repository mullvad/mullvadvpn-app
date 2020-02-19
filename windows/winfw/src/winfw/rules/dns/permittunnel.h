#pragma once

#include <winfw/rules/ifirewallrule.h>
#include <libwfp/ipaddress.h>
#include <vector>
#include <string>

namespace rules::dns
{

class PermitTunnel : public IFirewallRule
{
public:

	PermitTunnel(const std::wstring &tunnelInterfaceAlias, const std::vector<wfp::IpAddress> &hosts);
	
	bool apply(IObjectInstaller &objectInstaller) override;

private:

	const std::wstring m_tunnelInterfaceAlias;
	std::vector<wfp::IpAddress> m_hostsIpv4;
	std::vector<wfp::IpAddress> m_hostsIpv6;
};

}
