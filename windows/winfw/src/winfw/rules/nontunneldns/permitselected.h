#pragma once

#include <winfw/rules/ifirewallrule.h>
#include <libwfp/ipaddress.h>
#include <vector>
#include <optional>
#include <string>

namespace rules::nontunneldns
{

class PermitSelected : public IFirewallRule
{
public:

	//
	// The alias argument has to be optional for when the relay is connected on port 53.
	// At this point in time there's no tunnel yet.
	//
	PermitSelected(std::optional<std::wstring> tunnelInterfaceAlias, const std::vector<wfp::IpAddress> &hosts);
	
	bool apply(IObjectInstaller &objectInstaller) override;

private:

	const std::optional<std::wstring> m_tunnelInterfaceAlias;
	std::vector<wfp::IpAddress> m_hostsIpv4;
	std::vector<wfp::IpAddress> m_hostsIpv6;
};

}
