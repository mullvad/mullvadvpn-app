#pragma once

#include "ifirewallrule.h"

namespace rules
{

class PermitLan : public IFirewallRule
{
public:

	PermitLan() = default;
	~PermitLan() = default;
	
	bool apply(IObjectInstaller &objectInstaller) override;
};

}
