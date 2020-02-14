#pragma once

#include <winfw/rules/ifirewallrule.h>
#include <string>

namespace rules::baseline
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
