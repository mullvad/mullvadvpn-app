#pragma once

#include "guidhash.h"
#include <guiddef.h>
#include <unordered_set>
#include <map>

class MullvadGuids
{
public:

	MullvadGuids() = delete;

	static const GUID &Provider();
	static const GUID &SublayerBaseline();
	static const GUID &SublayerDns();

	//
	// Filter identifiers
	// Naming convention: Filter_sublayer_rule_filter
	//

	static const GUID &Filter_Baseline_BlockAll_Outbound_Ipv4();
	static const GUID &Filter_Baseline_BlockAll_Inbound_Ipv4();
	static const GUID &Filter_Baseline_BlockAll_Outbound_Ipv6();
	static const GUID &Filter_Baseline_BlockAll_Inbound_Ipv6();

	static const GUID &Filter_Baseline_PermitLan_Outbound_Ipv4();
	static const GUID &Filter_Baseline_PermitLan_Outbound_Multicast_Ipv4();
	static const GUID &Filter_Baseline_PermitLan_Outbound_Ipv6();
	static const GUID &Filter_Baseline_PermitLan_Outbound_Multicast_Ipv6();

	static const GUID &Filter_Baseline_PermitLanService_Inbound_Ipv4();
	static const GUID &Filter_Baseline_PermitLanService_Inbound_Ipv6();

	static const GUID &Filter_Baseline_PermitLoopback_Outbound_Ipv4();
	static const GUID &Filter_Baseline_PermitLoopback_Inbound_Ipv4();
	static const GUID &Filter_Baseline_PermitLoopback_Outbound_Ipv6();
	static const GUID &Filter_Baseline_PermitLoopback_Inbound_Ipv6();

	static const GUID &Filter_Baseline_PermitDhcp_Outbound_Request_Ipv4();
	static const GUID &Filter_Baseline_PermitDhcp_Inbound_Response_Ipv4();
	static const GUID &Filter_Baseline_PermitDhcp_Outbound_Request_Ipv6();
	static const GUID &Filter_Baseline_PermitDhcp_Inbound_Response_Ipv6();

	static const GUID &Filter_Baseline_PermitDhcpServer_Inbound_Request_Ipv4();
	static const GUID &Filter_Baseline_PermitDhcpServer_Outbound_Response_Ipv4();

	static const GUID &Filter_Baseline_PermitVpnTunnel_Outbound_Ipv4_1();
	static const GUID &Filter_Baseline_PermitVpnTunnel_Outbound_Ipv6_1();
	static const GUID &Filter_Baseline_PermitVpnTunnel_Outbound_Ipv4_2();
	static const GUID &Filter_Baseline_PermitVpnTunnel_Outbound_Ipv6_2();
	static const GUID &Filter_Baseline_PermitVpnTunnel_ExitIp();
	static const GUID &Filter_Baseline_PermitVpnTunnel_BlockExitIp();

	static const GUID &Filter_Baseline_PermitVpnTunnelService_Ipv4_1();
	static const GUID &Filter_Baseline_PermitVpnTunnelService_Ipv6_1();
	static const GUID &Filter_Baseline_PermitVpnTunnelService_Ipv4_2();
	static const GUID &Filter_Baseline_PermitVpnTunnelService_Ipv6_2();
	static const GUID &Filter_Baseline_PermitVpnTunnelService_ExitIp();
	static const GUID &Filter_Baseline_PermitVpnTunnelService_BlockExitIp();

	static const GUID &Filter_Baseline_PermitNdp_Outbound_Router_Solicitation();
	static const GUID &Filter_Baseline_PermitNdp_Inbound_Router_Advertisement();
	static const GUID &Filter_Baseline_PermitNdp_Outbound_Neighbor_Solicitation();
	static const GUID &Filter_Baseline_PermitNdp_Inbound_Neighbor_Solicitation();
	static const GUID &Filter_Baseline_PermitNdp_Outbound_Neighbor_Advertisement();
	static const GUID &Filter_Baseline_PermitNdp_Inbound_Neighbor_Advertisement();
	static const GUID &Filter_Baseline_PermitNdp_Inbound_Redirect();

	static const GUID &Filter_Baseline_PermitDns_Outbound_Ipv4();
	static const GUID &Filter_Baseline_PermitDns_Outbound_Ipv6();

	static const GUID &Filter_Dns_BlockAll_Outbound_Ipv4();
	static const GUID &Filter_Dns_BlockAll_Outbound_Ipv6();
	static const GUID &Filter_Dns_PermitNonTunnel_Outbound_Ipv4();
	static const GUID &Filter_Dns_PermitNonTunnel_Outbound_Ipv6();
	static const GUID &Filter_Dns_PermitTunnel_Outbound_Ipv4();
	static const GUID &Filter_Dns_PermitTunnel_Outbound_Ipv6();
	static const GUID &Filter_Dns_PermitLoopback_Outbound_Ipv4();
	static const GUID &Filter_Dns_PermitLoopback_Outbound_Ipv6();

	//
	// Persistent and boot-time filters
	//

	static const GUID &ProviderPersistent();
	static const GUID &SublayerPersistent();

	static const GUID &Filter_Boottime_BlockAll_Inbound_Ipv4();
	static const GUID &Filter_Boottime_BlockAll_Outbound_Ipv4();
	static const GUID &Filter_Boottime_BlockAll_Inbound_Ipv6();
	static const GUID &Filter_Boottime_BlockAll_Outbound_Ipv6();

	static const GUID &Filter_Persistent_BlockAll_Inbound_Ipv4();
	static const GUID &Filter_Persistent_BlockAll_Outbound_Ipv4();
	static const GUID &Filter_Persistent_BlockAll_Inbound_Ipv6();
	static const GUID &Filter_Persistent_BlockAll_Outbound_Ipv6();
};
