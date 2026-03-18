#pragma once

#include <winfw/rules/ifirewallrule.h>

namespace rules::baseline
{

class BlockAll : public IFirewallRule
{
public:

	BlockAll(const GUID &sublayerKey);
	~BlockAll() = default;

	bool apply(IObjectInstaller &objectInstaller) override;

private:

	const GUID m_sublayerKey;
};

}
