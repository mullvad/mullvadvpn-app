#pragma once

#include <winfw/rules/ifirewallrule.h>

namespace rules::baseline
{

class PermitDhcp : public IFirewallRule
{
public:

	PermitDhcp(const GUID &sublayerKey);
	~PermitDhcp() = default;

	bool apply(IObjectInstaller &objectInstaller) override;

private:

	const GUID m_sublayerKey;

	bool applyIpv4(IObjectInstaller &objectInstaller) const;
	bool applyIpv6(IObjectInstaller &objectInstaller) const;
};

}
