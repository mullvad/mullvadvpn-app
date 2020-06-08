#pragma once

#include <winfw/rules/ifirewallrule.h>
#include <libwfp/ipaddress.h>
#include <vector>
#include <optional>
#include <string>

//
// N.B. This rule must only be used for "custom DNS".
// Connecting to a relay on port 53 is supported using a different rule.
//

namespace rules::dns
{

class PermitNonTunnel : public IFirewallRule
{
public:

	//
	// The tunnel alias is optional so this rule can be applied even
	// when no tunnel exists.
	//
	// If a tunnel does exist, the alias must be provided.
	//
	PermitNonTunnel(std::optional<std::wstring> tunnelInterfaceAlias, const std::vector<wfp::IpAddress> &hosts);
	
	bool apply(IObjectInstaller &objectInstaller) override;

private:

	const std::optional<std::wstring> m_tunnelInterfaceAlias;
	std::vector<wfp::IpAddress> m_hostsIpv4;
	std::vector<wfp::IpAddress> m_hostsIpv6;
};

}
