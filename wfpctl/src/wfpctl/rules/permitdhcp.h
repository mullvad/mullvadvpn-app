#pragma once

#include "ifirewallrule.h"

namespace rules
{

class PermitDhcp : public IFirewallRule
{
public:

	PermitDhcp() = default;
	~PermitDhcp() = default;
	
	bool apply(IObjectInstaller &objectInstaller) override;
};

}
