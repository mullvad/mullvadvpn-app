#pragma once

#include <winfw/rules/ifirewallrule.h>
#include <libwfp/ipaddress.h>
#include <string>
#include <optional>
#include <vector>

namespace rules::baseline
{

class PermitPing : public IFirewallRule
{
public:

	PermitPing(std::optional<std::wstring> interfaceAlias, const std::vector<wfp::IpAddress> &hosts);

	bool apply(IObjectInstaller &objectInstaller) override;

private:

	const std::optional<std::wstring> m_interfaceAlias;
	std::vector<wfp::IpAddress> m_hostsIpv4;
	std::vector<wfp::IpAddress> m_hostsIpv6;

	bool applyIcmpv4(IObjectInstaller &objectInstaller) const;
	bool applyIcmpv6(IObjectInstaller &objectInstaller) const;
};

}
