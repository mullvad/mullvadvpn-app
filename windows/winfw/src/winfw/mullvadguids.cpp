#include "stdafx.h"
#include "mullvadguids.h"
#include <algorithm>
#include <iterator>

//static
WfpObjectRegistry MullvadGuids::BuildRegistry()
{
	const auto detailedRegistry = DetailedRegistry();
	using ValueType = decltype(detailedRegistry)::const_reference;

	std::unordered_set<GUID> registry;

	std::transform(detailedRegistry.begin(), detailedRegistry.end(), std::inserter(registry, registry.end()), [](ValueType value)
	{
		return value.second;
	});

	return registry;
}

//static
DetailedWfpObjectRegistry MullvadGuids::BuildDetailedRegistry()
{
	std::multimap<WfpObjectType, GUID> registry;

	registry.insert(std::make_pair(WfpObjectType::Provider, Provider()));
	registry.insert(std::make_pair(WfpObjectType::Sublayer, SublayerWhitelist()));
	registry.insert(std::make_pair(WfpObjectType::Sublayer, SublayerBlacklist()));
	registry.insert(std::make_pair(WfpObjectType::Filter, FilterBlockAll_Outbound_Ipv4()));
	registry.insert(std::make_pair(WfpObjectType::Filter, FilterBlockAll_Inbound_Ipv4()));
	registry.insert(std::make_pair(WfpObjectType::Filter, FilterBlockAll_Outbound_Ipv6()));
	registry.insert(std::make_pair(WfpObjectType::Filter, FilterBlockAll_Inbound_Ipv6()));
	registry.insert(std::make_pair(WfpObjectType::Filter, FilterPermitLan_Outbound_Ipv4()));
	registry.insert(std::make_pair(WfpObjectType::Filter, FilterPermitLan_Outbound_Multicast_Ipv4()));
	registry.insert(std::make_pair(WfpObjectType::Filter, FilterPermitLan_Outbound_Ipv6()));
	registry.insert(std::make_pair(WfpObjectType::Filter, FilterPermitLan_Outbound_Multicast_Ipv6()));
	registry.insert(std::make_pair(WfpObjectType::Filter, FilterPermitLanService_Inbound_Ipv4()));
	registry.insert(std::make_pair(WfpObjectType::Filter, FilterPermitLanService_Inbound_Ipv6()));
	registry.insert(std::make_pair(WfpObjectType::Filter, FilterPermitLoopback_Outbound_Ipv4()));
	registry.insert(std::make_pair(WfpObjectType::Filter, FilterPermitLoopback_Inbound_Ipv4()));
	registry.insert(std::make_pair(WfpObjectType::Filter, FilterPermitLoopback_Outbound_Ipv6()));
	registry.insert(std::make_pair(WfpObjectType::Filter, FilterPermitLoopback_Inbound_Ipv6()));
	registry.insert(std::make_pair(WfpObjectType::Filter, FilterPermitDhcp_Outbound_Request_Ipv4()));
	registry.insert(std::make_pair(WfpObjectType::Filter, FilterPermitDhcp_Inbound_Response_Ipv4()));
	registry.insert(std::make_pair(WfpObjectType::Filter, FilterPermitDhcp_Outbound_Request_Ipv6()));
	registry.insert(std::make_pair(WfpObjectType::Filter, FilterPermitDhcp_Inbound_Response_Ipv6()));
	registry.insert(std::make_pair(WfpObjectType::Filter, FilterPermitDhcpServer_Inbound_Request_Ipv4()));
	registry.insert(std::make_pair(WfpObjectType::Filter, FilterPermitDhcpServer_Outbound_Response_Ipv4()));
	registry.insert(std::make_pair(WfpObjectType::Filter, FilterPermitVpnRelay()));
	registry.insert(std::make_pair(WfpObjectType::Filter, FilterPermitVpnTunnel_Outbound_Ipv4()));
	registry.insert(std::make_pair(WfpObjectType::Filter, FilterPermitVpnTunnel_Outbound_Ipv6()));
	registry.insert(std::make_pair(WfpObjectType::Filter, FilterRestrictDns_Outbound_Ipv4()));
	registry.insert(std::make_pair(WfpObjectType::Filter, FilterRestrictDns_Outbound_Tunnel_Ipv4()));
	registry.insert(std::make_pair(WfpObjectType::Filter, FilterRestrictDns_Outbound_Ipv6()));
	registry.insert(std::make_pair(WfpObjectType::Filter, FilterRestrictDns_Outbound_Tunnel_Ipv6()));
	registry.insert(std::make_pair(WfpObjectType::Filter, FilterPermitVpnTunnelService_Ipv4()));
	registry.insert(std::make_pair(WfpObjectType::Filter, FilterPermitVpnTunnelService_Ipv6()));
	registry.insert(std::make_pair(WfpObjectType::Filter, FilterPermitNdp_Outbound_Router_Solicitation()));
	registry.insert(std::make_pair(WfpObjectType::Filter, FilterPermitNdp_Inbound_Router_Advertisement()));
	registry.insert(std::make_pair(WfpObjectType::Filter, FilterPermitNdp_Inbound_Redirect()));
	registry.insert(std::make_pair(WfpObjectType::Filter, FilterPermitPing_Outbound_Icmpv4()));
	registry.insert(std::make_pair(WfpObjectType::Filter, FilterPermitPing_Outbound_Icmpv6()));

	return registry;
}

//static
const WfpObjectRegistry &MullvadGuids::Registry()
{
	static auto registry = BuildRegistry();	// TODO: Thread safety.
	return registry;
}

//static
const DetailedWfpObjectRegistry &MullvadGuids::DetailedRegistry()
{
	static auto registry = BuildDetailedRegistry();	// TODO: Thread safety.
	return registry;
}

//static
const GUID &MullvadGuids::Provider()
{
	static const GUID g =
	{
		0x21e1dab8,
		0xb9db,
		0x43c0,
		{ 0xb3, 0x43, 0xeb, 0x93, 0x65, 0xc7, 0xbd, 0xd2 }
	};

	return g;
}

//static
const GUID &MullvadGuids::SublayerWhitelist()
{
	static const GUID g =
	{
		0x11d1a31a,
		0xd7fa,
		0x469b,
		{ 0xbc, 0x21, 0xcc, 0xe9, 0x2e, 0x35, 0xfe, 0x90 }
	};

	return g;
}

//static
const GUID &MullvadGuids::SublayerBlacklist()
{
	static const GUID g =
	{
		0x843b74f0,
		0xb499,
		0x499a,
		{ 0xac, 0xe3, 0xf9, 0xee, 0xa2, 0x4, 0x89, 0xc1 }
	};

	return g;
}

//static
const GUID &MullvadGuids::FilterBlockAll_Outbound_Ipv4()
{
	static const GUID g =
	{
		0xa81c5411,
		0xfd0,
		0x43a9,
		{ 0xa9, 0xbe, 0x31, 0x3f, 0x29, 0x9d, 0xe6, 0x4f }
	};

	return g;
}

//static
const GUID &MullvadGuids::FilterBlockAll_Inbound_Ipv4()
{
	static const GUID g =
	{
		0x86d07155,
		0x885f,
		0x409a,
		{ 0x8f, 0x22, 0x1, 0x9f, 0x87, 0x7a, 0xe4, 0x9 }
	};

	return g;
}

//static
const GUID &MullvadGuids::FilterBlockAll_Outbound_Ipv6()
{
	static const GUID g =
	{
		0x8ae5c389,
		0xd604,
		0x43df,
		{ 0x87, 0x4a, 0x5c, 0x86, 0x76, 0xc9, 0xc2, 0xb8 }
	};

	return g;
}

//static
const GUID &MullvadGuids::FilterBlockAll_Inbound_Ipv6()
{
	static const GUID g =
	{
		0x18b8c1d2,
		0x5910,
		0x4b51,
		{ 0xa5, 0x48, 0x1e, 0xfc, 0xd5, 0x4b, 0x63, 0xe9 }
	};

	return g;
}


//static
const GUID &MullvadGuids::FilterPermitLan_Outbound_Ipv4()
{
	static const GUID g =
	{
		0xb012b076,
		0x80d1,
		0x4628,
		{ 0x8d, 0x7b, 0xa5, 0x58, 0x8, 0xd8, 0xdc, 0xa4 }
	};

	return g;
}

//static
const GUID &MullvadGuids::FilterPermitLan_Outbound_Multicast_Ipv4()
{
	static const GUID g =
	{
		0xea5e136b,
		0xd951,
		0x4263,
		{ 0x99, 0xd8, 0x85, 0xc3, 0xf6, 0x4b, 0xda, 0xe9 }
	};

	return g;
}

//static
const GUID &MullvadGuids::FilterPermitLan_Outbound_Ipv6()
{
	static const GUID g =
	{
		0xacb22069,
		0xed33,
		0x4c6d,
		{ 0x9b, 0xc8, 0xcd, 0xfa, 0x6a, 0x1a, 0x10, 0x35 }
	};

	return g;
}

//static
const GUID &MullvadGuids::FilterPermitLan_Outbound_Multicast_Ipv6()
{
	static const GUID g =
	{
		0xb63d89ec,
		0xe145,
		0x4e29,
		{ 0x90, 0x87, 0xa7, 0x9b, 0xd6, 0xfc, 0x8b, 0x29 }
	};

	return g;
}

//static
const GUID &MullvadGuids::FilterPermitLanService_Inbound_Ipv4()
{
	static const GUID g =
	{
		0x5849930,
		0x40ae,
		0x41e4,
		{ 0x81, 0x68, 0x21, 0x94, 0x89, 0x8e, 0x6f, 0x8c }
	};

	return g;
}

//static
const GUID &MullvadGuids::FilterPermitLanService_Inbound_Ipv6()
{
	static const GUID g =
	{
		0xe8122820,
		0xe138,
		0x46b0,
		{ 0x96, 0x6f, 0x68, 0xa0, 0x6, 0xa2, 0xb5, 0xa2 }
	};

	return g;
}

//static
const GUID &MullvadGuids::FilterPermitLoopback_Outbound_Ipv4()
{
	static const GUID g =
	{
		0xd9ff592d,
		0xbe46,
		0x49fb,
		{ 0x97, 0xec, 0x71, 0x1, 0x3c, 0x12, 0xb8, 0x30 }
	};

	return g;
}

//static
const GUID &MullvadGuids::FilterPermitLoopback_Inbound_Ipv4()
{
	static const GUID g =
	{
		0xb8efb500,
		0xc51,
		0x4550,
		{ 0xbf, 0x5c, 0x48, 0x54, 0xa6, 0xc8, 0x48, 0xb9 }
	};

	return g;
}

//static
const GUID &MullvadGuids::FilterPermitLoopback_Outbound_Ipv6()
{
	static const GUID g =
	{
		0x764d4944,
		0x8a1e,
		0x4d96,
		{ 0xbf, 0xf0, 0x8d, 0xa6, 0x4f, 0x31, 0x44, 0xa2 }
	};

	return g;
}

//static
const GUID &MullvadGuids::FilterPermitLoopback_Inbound_Ipv6()
{
	static const GUID g =
	{
		0xbad325b0,
		0x736c,
		0x4e67,
		{ 0x8b, 0x37, 0x62, 0xb2, 0xdb, 0xe7, 0xd6, 0xeb }
	};

	return g;
}

//static
const GUID &MullvadGuids::FilterPermitDhcp_Outbound_Request_Ipv4()
{
	static const GUID g =
	{
		0x6cf1687b,
		0x35e9,
		0x4d18,
		{ 0xa2, 0x3, 0xb2, 0x6b, 0x71, 0xa9, 0x5f, 0x8d }
	};

	return g;
}

//static
const GUID &MullvadGuids::FilterPermitDhcp_Inbound_Response_Ipv4()
{
	static const GUID g =
	{
		0x2db298d7,
		0x4108,
		0x47ff,
		{ 0x85, 0x99, 0xaf, 0xa5, 0xcb, 0x95, 0x9c, 0x25 }
	};

	return g;
}

//static
const GUID &MullvadGuids::FilterPermitDhcp_Outbound_Request_Ipv6()
{
	static const GUID g =
	{
		0x67bd69b0,
		0x522d,
		0x4631,
		{ 0x9a, 0x8f, 0x1c, 0xee, 0xdf, 0x64, 0xb7, 0x2b }
	};

	return g;
}

//static
const GUID &MullvadGuids::FilterPermitDhcp_Inbound_Response_Ipv6()
{
	static const GUID g =
	{
		0x40dcfb6d,
		0x2ee,
		0x4531,
		{ 0x86, 0x61, 0xc4, 0xc8, 0xa4, 0x3a, 0xf4, 0x23 }
	};

	return g;
}

//static
const GUID &MullvadGuids::FilterPermitDhcpServer_Inbound_Request_Ipv4()
{
	static const GUID g =
	{
		0xa6c98ac3,
		0xe06,
		0x4fd2,
		{ 0xb4, 0x5e, 0xb7, 0xef, 0x67, 0x4, 0x43, 0xbc }
	};

	return g;
}

//static
const GUID &MullvadGuids::FilterPermitDhcpServer_Outbound_Response_Ipv4()
{
	static const GUID g =
	{
		0x57006c23,
		0xc21f,
		0x4d23,
		{ 0x88, 0xf, 0x5a, 0x9d, 0x94, 0x6b, 0xc2, 0xf3 }
	};

	return g;
}

//static
const GUID &MullvadGuids::FilterPermitVpnRelay()
{
	static const GUID g =
	{
		0x160c205d,
		0xdb40,
		0x4f79,
		{ 0x90, 0x6d, 0xfd, 0xa1, 0xe1, 0xc1, 0x8a, 0x70 }
	};

	return g;
}

//static
const GUID &MullvadGuids::FilterPermitVpnTunnel_Outbound_Ipv4()
{
	static const GUID g =
	{
		0xdfdcbb76,
		0x2284,
		0x4b03,
		{ 0x93, 0x4e, 0x93, 0xe5, 0xd3, 0x84, 0x8c, 0xf1 }
	};

	return g;
}

//static
const GUID &MullvadGuids::FilterPermitVpnTunnel_Outbound_Ipv6()
{
	static const GUID g =
	{
		0x9b1fa7d,
		0x843b,
		0x4946,
		{ 0xa6, 0x2, 0x90, 0x4, 0x26, 0x2a, 0xb8, 0x6b }
	};

	return g;
}

//static
const GUID &MullvadGuids::FilterRestrictDns_Outbound_Ipv4()
{
	static const GUID g =
	{
		0xc0792b44,
		0xfc3c,
		0x42e8,
		{ 0xa6, 0x60, 0x25, 0x4b, 0xd0, 0x4, 0xb1, 0x9d }
	};

	return g;
}

//static
const GUID &MullvadGuids::FilterRestrictDns_Outbound_Tunnel_Ipv4()
{
	static const GUID g =
	{
		0x790445dc,
		0xb23e,
		0x4ab4,
		{ 0x8e, 0x2f, 0xc7, 0x6, 0x55, 0x5f, 0x94, 0xff }
	};

	return g;
}

//static
const GUID &MullvadGuids::FilterRestrictDns_Outbound_Ipv6()
{
	static const GUID g =
	{
		0xcde477eb,
		0x2d8a,
		0x45b8,
		{ 0x9a, 0x3e, 0x9a, 0xa3, 0xbe, 0x4d, 0xe2, 0xb4 }
	};

	return g;
}

//static
const GUID &MullvadGuids::FilterRestrictDns_Outbound_Tunnel_Ipv6()
{
	static const GUID g =
	{
		0xacc90d87,
		0xab77,
		0x4cf4,
		{ 0x84, 0xee, 0x1d, 0x68, 0x95, 0xf0, 0x66, 0xc2 }
	};

	return g;
}

//static
const GUID &MullvadGuids::FilterPermitVpnTunnelService_Ipv4()
{
	static const GUID g =
	{
		0xf11a9ab4,
		0x3dd6,
		0x4cd9,
		{ 0x9d, 0x95, 0xb0, 0x36, 0x22, 0x71, 0x6b, 0x3d }
	};

	return g;
}

//static
const GUID &MullvadGuids::FilterPermitVpnTunnelService_Ipv6()
{
	static const GUID g =
	{
		0xe902e448,
		0x1845,
		0x42e5,
		{ 0xad, 0xf3, 0x33, 0xb2, 0x7a, 0xd, 0x5d, 0x38 }
	};

	return g;
}

//static
const GUID &MullvadGuids::FilterPermitNdp_Outbound_Router_Solicitation()
{
	static const GUID g =
	{
		0xbc5a85e4,
		0x5319,
		0x4224,
		{ 0x8a, 0x27, 0x53, 0xeb, 0x61, 0xef, 0x3b, 0x1 }
	};

	return g;
}

//static
const GUID &MullvadGuids::FilterPermitNdp_Inbound_Router_Advertisement()
{
	static const GUID g =
	{
		0x4d996f1d,
		0x4915,
		0x4a6a,
		{ 0xbd, 0xf5, 0xb5, 0x1a, 0x2d, 0xbc, 0xb8, 0xe9 }
	};

	return g;
}

//static
const GUID &MullvadGuids::FilterPermitNdp_Inbound_Redirect()
{
	static const GUID g =
	{
		0xcec23a8,
		0x4fdd,
		0x4a96,
		{ 0xae, 0xba, 0x33, 0xd2, 0xa7, 0xf, 0x85, 0x22 }
	};

	return g;
}

//static
const GUID &MullvadGuids::FilterPermitPing_Outbound_Icmpv4()
{
	static const GUID g =
	{
		0x2ecf7ff7,
		0xc951,
		0x4056,
		{ 0xb0, 0xf7, 0x40, 0xa4, 0x5c, 0x7e, 0xb4, 0xc2 }
	};

	return g;
}

//static
const GUID &MullvadGuids::FilterPermitPing_Outbound_Icmpv6()
{
	static const GUID g =
	{
		0x3deb8cab,
		0x1edb,
		0x4aa1,
		{ 0xb2, 0x73, 0xec, 0x61, 0x4f, 0x50, 0xdc, 0x13 }
	};

	return g;
}
