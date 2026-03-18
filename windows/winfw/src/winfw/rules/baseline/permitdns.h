#pragma once

#include <winfw/rules/ifirewallrule.h>

namespace rules::baseline
{

class PermitDns : public IFirewallRule
{
public:

	PermitDns(const GUID &sublayerKey);
	~PermitDns() = default;

	bool apply(IObjectInstaller& objectInstaller) override;

private:

	const GUID m_sublayerKey;
};

}
