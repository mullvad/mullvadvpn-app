#pragma once

#include <winfw/rules/ifirewallrule.h>

namespace rules::dns
{

class BlockAll : public IFirewallRule
{
public:

	bool apply(IObjectInstaller &objectInstaller) override;
};

}
