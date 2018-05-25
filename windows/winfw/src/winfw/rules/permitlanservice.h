#pragma once

#include "ifirewallrule.h"

namespace rules
{

class PermitLanService : public IFirewallRule
{
public:

	PermitLanService() = default;
	~PermitLanService() = default;
	
	bool apply(IObjectInstaller &objectInstaller) override;
};

}
