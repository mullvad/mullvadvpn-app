#pragma once

#include <winfw/rules/ifirewallrule.h>
#include <string>

namespace rules::baseline
{

class PermitVpnTunnelService : public IFirewallRule
{
public:

	PermitVpnTunnelService(const std::wstring &tunnelInterfaceAlias);

	bool apply(IObjectInstaller &objectInstaller) override;

private:

	const std::wstring m_tunnelInterfaceAlias;
};

}
