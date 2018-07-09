#pragma once

#include "ifirewallrule.h"
#include <string>

namespace rules
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
