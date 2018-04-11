#pragma once

#include "ifirewallrule.h"
#include <string>

namespace rules
{

class PermitVpnTunnel : public IFirewallRule
{
public:

	PermitVpnTunnel(const std::wstring &tunnelInterfaceAlias);
	
	bool apply(IObjectInstaller &objectInstaller) override;

private:

	const std::wstring m_tunnelInterfaceAlias;
};

}
