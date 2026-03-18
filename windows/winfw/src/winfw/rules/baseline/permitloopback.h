#pragma once

#include <winfw/rules/ifirewallrule.h>

namespace rules::baseline
{

class PermitLoopback : public IFirewallRule
{
public:

	PermitLoopback(const GUID &sublayerKey);
	~PermitLoopback() = default;

	bool apply(IObjectInstaller &objectInstaller) override;

private:

	const GUID m_sublayerKey;
};

}
