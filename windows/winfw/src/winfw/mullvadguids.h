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
	static const GUID &FilterBlockAll_Inbound_Ipv4();
	static const GUID &FilterBlockAll_Outbound_Ipv6();
	static const GUID &FilterBlockAll_Inbound_Ipv6();

	static const GUID &FilterPermitLan_Outbound_Ipv4();
	static const GUID &FilterPermitLan_Outbound_Multicast_Ipv4();
	static const GUID &FilterPermitLan_Outbound_Ipv6();
	static const GUID &FilterPermitLan_Outbound_Multicast_Ipv6();

	static const GUID &FilterPermitLanService_Inbound_Ipv4();
	static const GUID &FilterPermitLanService_Inbound_Ipv6();

	static const GUID &FilterPermitLoopback_Outbound_Ipv4();
	static const GUID &FilterPermitLoopback_Inbound_Ipv4();
	static const GUID &FilterPermitLoopback_Outbound_Ipv6();
	static const GUID &FilterPermitLoopback_Inbound_Ipv6();

	static const GUID &FilterPermitDhcp_Outbound_Request_Ipv4();
	static const GUID &FilterPermitDhcp_Inbound_Response_Ipv4();
	static const GUID &FilterPermitDhcp_Outbound_Request_Ipv6();
	static const GUID &FilterPermitDhcp_Inbound_Response_Ipv6();

	static const GUID &FilterPermitDhcpServer_Inbound_Request_Ipv4();
	static const GUID &FilterPermitDhcpServer_Outbound_Response_Ipv4();

	static const GUID &FilterPermitVpnRelay();

	static const GUID &FilterPermitVpnTunnel_Outbound_Ipv4();
	static const GUID &FilterPermitVpnTunnel_Outbound_Ipv6();

	static const GUID &FilterRestrictDns_Outbound_Ipv4();
	static const GUID &FilterRestrictDns_Outbound_Tunnel_Ipv4();
	static const GUID &FilterRestrictDns_Outbound_Ipv6();
	static const GUID &FilterRestrictDns_Outbound_Tunnel_Ipv6();

	static const GUID &FilterPermitVpnTunnelService_Ipv4();
	static const GUID &FilterPermitVpnTunnelService_Ipv6();

	static const GUID &FilterPermitNdp_Outbound_Router_Solicitation();
	static const GUID &FilterPermitNdp_Inbound_Router_Advertisement();
	static const GUID &FilterPermitNdp_Inbound_Redirect();

	static const GUID &FilterPermitPing_Outbound_Icmpv4();
	static const GUID &FilterPermitPing_Outbound_Icmpv6();
};
