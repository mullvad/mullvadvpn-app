#pragma once

#include <winfw/rules/ifirewallrule.h>
#include <libwfp/ipaddress.h>

namespace rules::baseline
{

class PermitVpnRelay : public IFirewallRule
{
public:

	enum class Protocol
	{
		Tcp,
		Udp
	};

	PermitVpnRelay(const wfp::IpAddress &relay, uint16_t relayPort, Protocol protocol);
	
	bool apply(IObjectInstaller &objectInstaller) override;

private:

	const wfp::IpAddress m_relay;
	const uint16_t m_relayPort;
	const Protocol m_protocol;
};

}
