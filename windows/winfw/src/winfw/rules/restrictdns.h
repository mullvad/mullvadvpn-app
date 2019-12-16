#pragma once

#include "ifirewallrule.h"
#include "libwfp/ipaddress.h"
#include <optional>
#include <string>
#include <cstdint>

namespace rules
{

class RestrictDns : public IFirewallRule
{
public:

	RestrictDns(const std::wstring &tunnelInterfaceAlias, const wfp::IpAddress v4DnsHost, std::optional<wfp::IpAddress> v6DnsHost, wfp::IpAddress relay, uint16_t relayPort);

	bool apply(IObjectInstaller &objectInstaller) override;

private:

	const std::wstring m_tunnelInterfaceAlias;
	const wfp::IpAddress m_v4DnsHost;
	const std::optional<wfp::IpAddress> m_v6DnsHost;
	const uint16_t m_relayPort;
	const wfp::IpAddress m_relayHost;

};

}
