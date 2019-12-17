#pragma once

#include "ifirewallrule.h"
#include "libwfp/ipaddress.h"
#include <optional>
#include <string>
#include <cstdint>

namespace rules
{

class PermitTunnelDns : public IFirewallRule
{
public:

	PermitTunnelDns::PermitTunnelDns(const std::wstring &tunnelInterfaceAlias, const wfp::IpAddress v4DnsHost, const std::optional<wfp::IpAddress> v6DnsHost);

	bool apply(IObjectInstaller &objectInstaller) override;

private:

	const std::wstring m_tunnelInterfaceAlias;
	const wfp::IpAddress m_v4DnsHost;
	const std::optional<wfp::IpAddress> m_v6DnsHost;

};

}
