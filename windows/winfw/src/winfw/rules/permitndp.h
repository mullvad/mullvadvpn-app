#pragma once

#include "ifirewallrule.h"

namespace rules
{

class PermitNdp : public IFirewallRule
{
public:

	PermitNdp() = default;
	~PermitNdp() = default;
	
	bool apply(IObjectInstaller &objectInstaller) override;
};

}
