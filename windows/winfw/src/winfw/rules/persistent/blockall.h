#pragma once

#include <winfw/rules/ifirewallrule.h>

namespace rules::persistent
{

class BlockAll : public IFirewallRule
{
public:

	BlockAll() = default;
	~BlockAll() = default;
	
	bool apply(IObjectInstaller &objectInstaller) override;
};

}
