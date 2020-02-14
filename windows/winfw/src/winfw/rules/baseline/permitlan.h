#pragma once

#include <winfw/rules/ifirewallrule.h>

namespace rules::baseline
{

class PermitLan : public IFirewallRule
{
public:

	PermitLan() = default;
	~PermitLan() = default;
	
	bool apply(IObjectInstaller &objectInstaller) override;

private:

	bool applyIpv4(IObjectInstaller &objectInstaller) const;
	bool applyIpv6(IObjectInstaller &objectInstaller) const;
};

}
