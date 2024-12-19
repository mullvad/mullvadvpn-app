#pragma once

#include <winfw/rules/ifirewallrule.h>
#include <winfw/winfw.h>
#include <libwfp/ipaddress.h>
#include <vector>
#include <string>

namespace rules::baseline
{

// Permit incoming ICMP/ICMP6 TimeExceeded packets to select clients.
class PermitIcmpTtl : public IFirewallRule
{
public:

	PermitIcmpTtl(const std::vector<std::wstring> &relayClients);
	bool apply(IObjectInstaller &objectInstaller) override;

private:

	const std::vector<std::wstring> m_relayClients;
};

}
