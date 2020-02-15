#pragma once

#include <winfw/rules/ifirewallrule.h>
#include <optional>
#include <string>

namespace rules::nontunneldns
{

class BlockAll : public IFirewallRule
{
public:

	BlockAll(std::optional<std::wstring> tunnelInterfaceAlias);
	
	bool apply(IObjectInstaller &objectInstaller) override;

private:

	const std::optional<std::wstring> m_tunnelInterfaceAlias;
};

}
