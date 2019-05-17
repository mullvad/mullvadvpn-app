#pragma once

#include "ifirewallrule.h"
#include <memory>

namespace rules
{

class PermitDhcpServer : public IFirewallRule
{
public:

	enum class Extent
	{
		All,
		IPv4Only,
		IPv6Only
	};

	static std::unique_ptr<PermitDhcpServer> WithExtent(Extent extent);

	bool apply(IObjectInstaller &objectInstaller) override;

private:

	PermitDhcpServer() = default;

	bool applyIpv4(IObjectInstaller &objectInstaller) const;
};

}
