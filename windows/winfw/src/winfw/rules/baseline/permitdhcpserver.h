#pragma once

#include <winfw/rules/ifirewallrule.h>
#include <memory>

namespace rules::baseline
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

	static std::unique_ptr<PermitDhcpServer> WithExtent(Extent extent, const GUID &sublayerKey);

	bool apply(IObjectInstaller &objectInstaller) override;

private:

	PermitDhcpServer(const GUID &sublayerKey);

	const GUID m_sublayerKey;

	bool applyIpv4(IObjectInstaller &objectInstaller) const;
};

}
