#pragma once

#include <winfw/rules/ifirewallrule.h>
#include <winfw/winfw.h>
#include <libwfp/ipaddress.h>
#include <string>
#include <optional>

namespace rules::multi
{

class PermitVpnRelay : public IFirewallRule
{
public:

	enum class Sublayer
	{
		Baseline,
		Dns
	};

	PermitVpnRelay
	(
		const wfp::IpAddress &relay,
		uint16_t relayPort,
		WinFwProtocol protocol,
		const std::optional<std::wstring> &relayClient,
		Sublayer sublayer
	);
	
	bool apply(IObjectInstaller &objectInstaller) override;

private:

	const wfp::IpAddress m_relay;
	const uint16_t m_relayPort;
	const WinFwProtocol m_protocol;
	const std::optional<std::wstring> m_relayClient;
	const Sublayer m_sublayer;
};

}
