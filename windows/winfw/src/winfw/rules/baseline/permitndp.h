#pragma once

#include <winfw/rules/ifirewallrule.h>

namespace rules::baseline
{

class PermitNdp : public IFirewallRule
{
public:

	PermitNdp() = default;
	~PermitNdp() = default;
	
	bool apply(IObjectInstaller &objectInstaller) override;
};

}
