#pragma once

#include <winfw/rules/ifirewallrule.h>
#include <winfw/rules/baseline/permitvpntunnel.h>
#include <winfw/winfw.h>
#include <libwfp/ipaddress.h>
#include <string>
#include <optional>

namespace rules::baseline
{

class PermitVpnTunnelService : public IFirewallRule
{
public:

	PermitVpnTunnelService(
		const std::vector<std::wstring> &relayClients,
		const std::wstring &tunnelInterfaceAlias,
		const std::optional<PermitVpnTunnel::Endpoints> &potentialEndpoints,
		const std::optional<wfp::IpAddress> &exitEndpointIp
	);

	bool apply(IObjectInstaller &objectInstaller) override;

private:
	bool AddEndpointFilter(const std::optional<PermitVpnTunnel::Endpoint> &endpoint, const GUID &ipv4Guid, const GUID &ipv6Guid, IObjectInstaller &objectInstaller);
	bool BlockNonRelayClientExit(const wfp::IpAddress &exitIp, IObjectInstaller &objectInstaller);

	const std::vector<std::wstring> m_relayClients;
	const std::wstring m_tunnelInterfaceAlias;
	const std::optional<PermitVpnTunnel::Endpoints> m_potentialEndpoints;
	const std::optional<wfp::IpAddress> m_exitEndpointIp;
};

}
