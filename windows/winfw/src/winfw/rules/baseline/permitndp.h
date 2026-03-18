#pragma once

#include <winfw/rules/ifirewallrule.h>

namespace rules::baseline
{

class PermitNdp : public IFirewallRule
{
public:

	PermitNdp(const GUID &sublayerKey);
	~PermitNdp() = default;

	bool apply(IObjectInstaller &objectInstaller) override;

private:

	const GUID m_sublayerKey;
};

}
