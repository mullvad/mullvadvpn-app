#pragma once

#include <winfw/rules/ifirewallrule.h>

namespace rules::baseline
{

class PermitDns : public IFirewallRule
{
public:

	PermitDns() = default;
	~PermitDns() = default;

	bool apply(IObjectInstaller& objectInstaller) override;
};

}
