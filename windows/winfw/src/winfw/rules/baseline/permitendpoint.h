#pragma once

#include <winfw/rules/ifirewallrule.h>
#include <libwfp/ipaddress.h>
#include <string>

namespace rules::baseline
{

class PermitEndpoint : public IFirewallRule
{
public:

	enum class Protocol
	{
		Tcp,
		Udp
	};

	PermitEndpoint
	(
		const wfp::IpAddress &address,
		uint16_t port,
		Protocol protocol
	);
	
	bool apply(IObjectInstaller &objectInstaller) override;

private:

	const wfp::IpAddress m_address;
	const uint16_t m_port;
	const Protocol m_protocol;
};

}
