#pragma once

#include <winfw/rules/ifirewallrule.h>
#include <libwfp/ipaddress.h>
#include <string>
#include <vector>

namespace rules::multi
{

class PermitVpnRelay : public IFirewallRule
{
public:

	enum class Protocol
	{
		Tcp,
		Udp
	};

	enum class Sublayer
	{
		Baseline,
		Dns
	};

	PermitVpnRelay
	(
		const wfp::IpAddress &relay,
		uint16_t relayPort,
		Protocol protocol,
		const std::vector<std::wstring> &approvedApplications,
		Sublayer sublayer
	);
	
	bool apply(IObjectInstaller &objectInstaller) override;

private:

	const wfp::IpAddress m_relay;
	const uint16_t m_relayPort;
	const Protocol m_protocol;
	const std::vector<std::wstring> m_approvedApplications;
	const Sublayer m_sublayer;
};

}
