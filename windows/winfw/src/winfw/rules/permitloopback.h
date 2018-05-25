#pragma once

#include "ifirewallrule.h"

namespace rules
{

class PermitLoopback : public IFirewallRule
{
public:

	PermitLoopback() = default;
	~PermitLoopback() = default;
	
	bool apply(IObjectInstaller &objectInstaller) override;
};

}
