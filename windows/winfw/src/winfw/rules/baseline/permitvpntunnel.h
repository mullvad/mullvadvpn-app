#pragma once

#include <winfw/rules/ifirewallrule.h>
#include <winfw/winfw.h>
#include <libwfp/ipaddress.h>
#include <string>
#include <optional>

namespace rules::baseline
{

class PermitVpnTunnel : public IFirewallRule
{
public:

	struct Endpoint {
		wfp::IpAddress ip;
		uint16_t port;
		WinFwProtocol protocol;
	};

	struct Endpoints {
		Endpoint endpoint1;
		std::optional<Endpoint> endpoint2;
	};

	PermitVpnTunnel(
		const std::optional<std::wstring> &relayClient,
		const std::wstring &tunnelInterfaceAlias,
		const std::optional<Endpoints> &potentialEndpoints,
		const std::optional<wfp::IpAddress> &exitEndpoint
	);
	
	bool apply(IObjectInstaller &objectInstaller) override;

private:
	bool AddEndpointFilter(const std::optional<Endpoint> &endpoint, const GUID &ipv4Guid, const GUID &ipv6Guid, IObjectInstaller &objectInstaller);
	bool BlockNonRelayClientExit(const wfp::IpAddress &exitIp, const std::wstring &relayClient, IObjectInstaller &objectInstaller);

	const std::optional<std::wstring> m_relayClient;
	const std::wstring m_tunnelInterfaceAlias;
	const std::optional<Endpoints> m_potentialEndpoints;
	const std::optional<wfp::IpAddress> m_exitEndpointIp;
};

}
