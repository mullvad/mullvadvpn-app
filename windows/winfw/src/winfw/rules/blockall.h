#pragma once

#include "ifirewallrule.h"

namespace rules
{

class BlockAll : public IFirewallRule
{
public:

	BlockAll() = default;
	~BlockAll() = default;
	
	bool apply(IObjectInstaller &objectInstaller) override;
};

}
