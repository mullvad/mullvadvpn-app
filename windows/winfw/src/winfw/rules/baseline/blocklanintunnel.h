#pragma once

#include <winfw/rules/ifirewallrule.h>
#include <string>

namespace rules::baseline
{

//
// Block private IP ranges on the tunnel device.
//
// Some exceptions are made for Mullvad IPs.
//
class BlockLanInTunnel : public IFirewallRule
{
public:

	BlockLanInTunnel(const std::wstring &tunnelInterfaceAlias);

	bool apply(IObjectInstaller &objectInstaller) override;

private:

	bool applyIpv4(IObjectInstaller &objectInstaller) const;
	bool applyIpv6(IObjectInstaller &objectInstaller) const;

	const std::wstring m_tunnelInterfaceAlias;
};

}
