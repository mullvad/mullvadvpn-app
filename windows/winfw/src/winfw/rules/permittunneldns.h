#pragma once

#include "ifirewallrule.h"
#include "libwfp/ipaddress.h"
#include <string>
#include <cstdint>

namespace rules
{

class PermitTunnelDns : public IFirewallRule
{
public:

	PermitTunnelDns(const std::wstring &tunnelInterfaceAlias, const std::vector<wfp::IpAddress> &dnsHosts);

	bool apply(IObjectInstaller &objectInstaller) override;

private:

	const std::wstring m_tunnelInterfaceAlias;
	std::vector<wfp::IpAddress> m_v4DnsHosts;
	std::vector<wfp::IpAddress> m_v6DnsHosts;

};

}
