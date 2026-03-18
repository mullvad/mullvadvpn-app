#pragma once

#include <winfw/rules/ifirewallrule.h>

namespace rules::dns
{

class BlockAll : public IFirewallRule
{
public:

	BlockAll(const GUID &sublayerKey);

	bool apply(IObjectInstaller &objectInstaller) override;

private:

	const GUID m_sublayerKey;
};

}
