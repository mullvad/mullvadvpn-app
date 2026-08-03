#pragma once

#include <winfw/rules/ifirewallrule.h>
#include <winfw/winfw.h>
#include <libwfp/ipaddress.h>
#include <cstdint>
#include <string>
#include <vector>

namespace rules::multi
{

//
// Permit outbound connections originating from a specific local endpoint,
// that is a local IP address, local port and transport protocol.
//
// This is the mirror image of `PermitEndpoint`, which is keyed on the *remote* endpoint.
// It is used to exclude ("bypass") individual sockets owned by the daemon from the firewall,
// without knowing which remote peers those sockets are going to talk to.
//
// If `localIp` is the unspecified address (`0.0.0.0` or `::`) the local IP condition is
// omitted, and the filter is installed at both the IPv4 and the IPv6 layer. A socket bound
// to the unspecified address may be dual-stack, and can therefore emit IPv4 traffic even
// though it was created as an IPv6 socket.
//
// The filters are installed in both the baseline and the DNS sublayer, because the remote
// port is unknown here: a peer listening on port 53 would otherwise be blocked by
// `dns::BlockAll`. Refer to the comment on `AppendSettingsRules` in fwcontext.cpp.
//
class PermitLocalEndpoint : public IFirewallRule
{
public:

	PermitLocalEndpoint
	(
		const wfp::IpAddress &localIp,
		uint16_t localPort,
		WinFwProtocol protocol,
		const std::vector<std::wstring> &clients
	);

	bool apply(IObjectInstaller &objectInstaller) override;

private:

	const wfp::IpAddress m_localIp;
	const uint16_t m_localPort;
	const WinFwProtocol m_protocol;
	const std::vector<std::wstring> m_clients;
};

}
