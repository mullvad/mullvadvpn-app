#pragma once

#include <winfw/rules/ifirewallrule.h>
#include <optional>
#include <string>

namespace rules::tunneldns
{

class BlockAll : public IFirewallRule
{
public:

	BlockAll(const std::wstring &tunnelInterfaceAlias);
	
	bool apply(IObjectInstaller &objectInstaller) override;

private:

	const std::wstring m_tunnelInterfaceAlias;
};

}
