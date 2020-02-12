#pragma once

#include "ifirewallrule.h"
#include "libwfp/ipaddress.h"
#include "winfw/winfw.h"
#include <optional>
#include <string>
#include <cstdint>

namespace rules
{

class RestrictDns : public IFirewallRule
{
public:

	struct DnsHosts
	{
		std::wstring tunnelInterfaceAlias;
		wfp::IpAddress v4DnsHost;
		std::optional<wfp::IpAddress> v6DnsHost;
	};

	RestrictDns(const std::optional<WinFwRelay> &relay, const std::optional<DnsHosts> &dnsHosts);

	bool apply(IObjectInstaller &objectInstaller) override;

private:

	std::optional<wfp::IpAddress> m_allowHost;
	const std::optional<DnsHosts> m_dnsHosts;

};

}
