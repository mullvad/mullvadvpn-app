#pragma once

#include <winfw/rules/ifirewallrule.h>
#include <libwfp/ipnetwork.h>
#include <vector>

namespace rules::baseline
{

//
// Permits outbound and inbound traffic to/from user-specified IP networks.
// This is used for split tunneling by IP/subnet, allowing user-specified networks
// to bypass the VPN firewall.
//
class PermitSplitTunnelSubnets : public IFirewallRule
{
public:

	PermitSplitTunnelSubnets(
		const std::vector<wfp::IpNetwork> &ipv4Subnets,
		const std::vector<wfp::IpNetwork> &ipv6Subnets
	);

	~PermitSplitTunnelSubnets() = default;

	bool apply(IObjectInstaller &objectInstaller) override;

private:

	bool applyIpv4Outbound(IObjectInstaller &objectInstaller) const;
	bool applyIpv4Inbound(IObjectInstaller &objectInstaller) const;
	bool applyIpv6Outbound(IObjectInstaller &objectInstaller) const;
	bool applyIpv6Inbound(IObjectInstaller &objectInstaller) const;

	std::vector<wfp::IpNetwork> m_ipv4Subnets;
	std::vector<wfp::IpNetwork> m_ipv6Subnets;
};

}
