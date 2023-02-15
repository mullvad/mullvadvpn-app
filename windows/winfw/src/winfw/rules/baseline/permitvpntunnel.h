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
        Endpoint entryEndpoint;
        std::optional<Endpoint> exitEndpoint;
    }

	PermitVpnTunnel(
		const std::wstring &tunnelInterfaceAlias,
		const std::optional<Endpoints> &potentialEndpoints,
	);
	
	bool apply(IObjectInstaller &objectInstaller) override;

private:

	const std::wstring m_tunnelInterfaceAlias;
	const std::optional<Endpoints> m_potentialEndpoints;
};

}
