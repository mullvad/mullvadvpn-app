#pragma once
#include <guiddef.h>

class MullvadGuids
{
public:

	MullvadGuids() = delete;

	static const GUID &Provider();
	static const GUID &SublayerWhitelist();
	static const GUID &SublayerBlacklist();

	static const GUID &FilterBlockAll_Outbound_Ipv4();
	static const GUID &FilterBlockAll_Outbound_Ipv6();
	static const GUID &FilterBlockAll_Inbound_Ipv4();
	static const GUID &FilterBlockAll_Inbound_Ipv6();

	static const GUID &FilterPermitLan_10_8();
	static const GUID &FilterPermitLan_172_16_12();
	static const GUID &FilterPermitLan_192_168_16();
	static const GUID &FilterPermitLan_Multicast();

	static const GUID &FilterPermitLanService_10_8();
	static const GUID &FilterPermitLanService_172_16_12();
	static const GUID &FilterPermitLanService_192_168_16();

	static const GUID &FilterPermitLoopback_Outbound_Ipv4();
	static const GUID &FilterPermitLoopback_Outbound_Ipv6();
	static const GUID &FilterPermitLoopback_Inbound_Ipv4();
	static const GUID &FilterPermitLoopback_Inbound_Ipv6();

	static const GUID &FilterPermitDhcp_Outbound_Request();
	static const GUID &FilterPermitDhcp_Inbound_Response();

	static const GUID &FilterPermitVpnRelay();

	static const GUID &FilterPermitVpnTunnel_Outbound_Ipv4();
	static const GUID &FilterPermitVpnTunnel_Outbound_Ipv6();

	static const GUID &FilterRestrictDns_Outbound_Ipv4();
	static const GUID &FilterRestrictDns_Outbound_Ipv6();
	static const GUID &FilterRestrictDns_Outbound_Tunnel_Ipv4();
	static const GUID &FilterRestrictDns_Outbound_Tunnel_Ipv6();
};
