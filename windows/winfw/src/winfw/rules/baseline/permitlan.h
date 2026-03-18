#pragma once

#include <winfw/rules/ifirewallrule.h>

namespace rules::baseline
{

class PermitLan : public IFirewallRule
{
public:

	PermitLan(const GUID &sublayerKey);
	~PermitLan() = default;

	bool apply(IObjectInstaller &objectInstaller) override;

private:

	const GUID m_sublayerKey;

	bool applyIpv4(IObjectInstaller &objectInstaller) const;
	bool applyIpv6(IObjectInstaller &objectInstaller) const;
};

}
