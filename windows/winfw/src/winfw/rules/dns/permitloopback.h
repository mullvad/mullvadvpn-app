#pragma once

#include <winfw/rules/ifirewallrule.h>

namespace rules::dns
{

class PermitLoopback : public IFirewallRule
{
public:

	PermitLoopback() = default;
	~PermitLoopback() = default;
	
	bool apply(IObjectInstaller &objectInstaller) override;
};

}
