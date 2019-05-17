#pragma once

#include "wfpobjecttype.h"
#include "guidhash.h"
#include <guiddef.h>
#include <unordered_set>
#include <map>

using WfpObjectRegistry = std::unordered_set<GUID>;
using DetailedWfpObjectRegistry = std::multimap<WfpObjectType, GUID>;

class MullvadGuids
{
	static WfpObjectRegistry BuildRegistry();
	static DetailedWfpObjectRegistry BuildDetailedRegistry();

public:

	static const WfpObjectRegistry &Registry();
	static const DetailedWfpObjectRegistry &DetailedRegistry();

	MullvadGuids() = delete;

	static const GUID &Provider();
	static const GUID &SublayerWhitelist();
	static const GUID &SublayerBlacklist();

	static const GUID &FilterBlockAll_Outbound_Ipv4();
	static const GUID &FilterBlockAll_Outbound_Ipv6();
	static const GUID &FilterBlockAll_Inbound_Ipv4();
	static const GUID &FilterBlockAll_Inbound_Ipv6();

	static const GUID &FilterPermitLan_Outbound_Ipv4();
	static const GUID &FilterPermitLan_Outbound_Multicast_Ipv4();
	static const GUID &FilterPermitLan_Outbound_Ipv6();
	static const GUID &FilterPermitLan_Outbound_Multicast_Ipv6();

	static const GUID &FilterPermitLanService_Inbound_Ipv4();
	static const GUID &FilterPermitLanService_Inbound_Ipv6();

	static const GUID &FilterPermitLoopback_Outbound_Ipv4();
	static const GUID &FilterPermitLoopback_Outbound_Ipv6();
	static const GUID &FilterPermitLoopback_Inbound_Ipv4();
	static const GUID &FilterPermitLoopback_Inbound_Ipv6();

	static const GUID &FilterPermitDhcpV4_Outbound_Request();
	static const GUID &FilterPermitDhcpV6_Outbound_Request();
	static const GUID &FilterPermitDhcpV4_Inbound_Response();
	static const GUID &FilterPermitDhcpV6_Inbound_Response();

	static const GUID &FilterPermitDhcpV4Server_Inbound_Request();
	static const GUID &FilterPermitDhcpV4Server_Outbound_Response();

	static const GUID &FilterPermitVpnRelay();

	static const GUID &FilterPermitVpnTunnel_Outbound_Ipv4();
	static const GUID &FilterPermitVpnTunnel_Outbound_Ipv6();

	static const GUID &FilterRestrictDns_Outbound_Ipv4();
	static const GUID &FilterRestrictDns_Outbound_Ipv6();
	static const GUID &FilterRestrictDns_Outbound_Tunnel_Ipv4();
	static const GUID &FilterRestrictDns_Outbound_Tunnel_Ipv6();

	static const GUID &FilterPermitVpnTunnelService_Ipv4();
	static const GUID &FilterPermitVpnTunnelService_Ipv6();

	static const GUID &FilterPermitNdp_Outbound_Router_Solicitation();
	static const GUID &FilterPermitNdp_Inbound_Router_Advertisement();
	static const GUID &FilterPermitNdp_Inbound_Redirect();
};
