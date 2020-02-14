#pragma once

#include <winfw/rules/ifirewallrule.h>

namespace rules::baseline
{

class PermitLanService : public IFirewallRule
{
public:

	PermitLanService() = default;
	~PermitLanService() = default;
	
	bool apply(IObjectInstaller &objectInstaller) override;

private:

	bool applyIpv4(IObjectInstaller &objectInstaller) const;
	bool applyIpv6(IObjectInstaller &objectInstaller) const;
};

}
