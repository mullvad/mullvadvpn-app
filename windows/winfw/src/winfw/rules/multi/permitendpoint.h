#pragma once

#include <winfw/rules/ifirewallrule.h>
#include <winfw/winfw.h>
#include <libwfp/ipaddress.h>
#include <string>

namespace rules::multi
{

class PermitEndpoint : public IFirewallRule
{
public:

	PermitEndpoint
	(
		const wfp::IpAddress &relay,
		uint16_t relayPort,
		WinFwProtocol protocol,
		const std::vector<std::wstring> &relayClients,
		const GUID &sublayerKey
	);

	bool apply(IObjectInstaller &objectInstaller) override;

private:

	const wfp::IpAddress m_relay;
	const uint16_t m_relayPort;
	const WinFwProtocol m_protocol;
	const std::vector<std::wstring> m_relayClients;
	const GUID m_sublayerKey;
};

}
