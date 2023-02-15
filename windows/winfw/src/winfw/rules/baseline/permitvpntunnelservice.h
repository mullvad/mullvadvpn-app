#pragma once

#include <winfw/rules/ifirewallrule.h>
#include <winfw/rules/baseline/permitvpntunnel.h>
#include <winfw/winfw.h>
#include <libwfp/ipaddress.h>
#include <string>
#include <vector>

namespace rules::baseline
{

class PermitVpnTunnelService : public IFirewallRule
{
public:

	PermitVpnTunnelService(
		const std::wstring &tunnelInterfaceAlias,
		const std::optional<PermitVpnTunnel::Endpoints> &potentialEndpoints
	);

	bool apply(IObjectInstaller &objectInstaller) override;

private:
    bool AddEndpointFilter(const std::optional<PermitVpnTunnel::Endpoint> &endpoint, GUID ipv4Guid, GUID ipv6Guid, wfp::FilterBuilder &filterBuilder, IObjectInstaller& objectInstaller);

	const std::wstring m_tunnelInterfaceAlias;
	const std::optional<PermitVpnTunnel::Endpoints> m_potentialEndpoints;
};

}
