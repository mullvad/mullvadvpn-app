#pragma once

#include "ifirewallrule.h"

namespace rules
{

class PermitDhcpServer : public IFirewallRule
{
public:

	PermitDhcpServer() = default;
	~PermitDhcpServer() = default;

	bool apply(IObjectInstaller &objectInstaller) override;

private:

	bool applyIpv4(IObjectInstaller &objectInstaller) const;
};

}
