#pragma once

#include <winfw/rules/ifirewallrule.h>
#include <winfw/winfw.h>
#include <libwfp/ipaddress.h>
#include <string>
#include <vector>

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

	PermitVpnTunnel(
		const std::wstring &tunnelInterfaceAlias,
		const std::vector<Endpoint> &endpoints
	);
	
	bool apply(IObjectInstaller &objectInstaller) override;

private:

	const std::wstring m_tunnelInterfaceAlias;
	const std::vector<Endpoint> m_tunnelEndpoint;
};

}
