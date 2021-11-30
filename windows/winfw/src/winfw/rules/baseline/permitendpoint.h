#pragma once

#include <winfw/rules/ifirewallrule.h>
#include <winfw/winfw.h>
#include <libwfp/ipaddress.h>
#include <vector>
#include <string>

namespace rules::baseline
{

class PermitEndpoint : public IFirewallRule
{
public:

	PermitEndpoint
	(
		const wfp::IpAddress &address,
		const std::vector<std::wstring> &clients,
		uint16_t port,
		WinFwProtocol protocol
	);
	
	bool apply(IObjectInstaller &objectInstaller) override;

private:

	const wfp::IpAddress m_address;
	const std::vector<std::wstring> m_clients;
	const uint16_t m_port;
	const WinFwProtocol m_protocol;
};

}
