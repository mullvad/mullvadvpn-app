#pragma once

#include "ifirewallrule.h"
#include <libwfp/ipaddress.h>
#include <string>
#include <optional>

namespace rules
{

class PermitPing : public IFirewallRule
{
public:

	PermitPing(const std::optional<std::wstring> &interfaceAlias, const wfp::IpAddress &host);

	bool apply(IObjectInstaller &objectInstaller) override;

private:

	const std::optional<std::wstring> m_interfaceAlias;
	const wfp::IpAddress m_host;

	bool applyIcmpv4(IObjectInstaller &objectInstaller) const;
	bool applyIcmpv6(IObjectInstaller &objectInstaller) const;
};

}
