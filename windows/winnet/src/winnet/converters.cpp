#include <stdafx.h>
#include "converters.h"
#include <libcommon/error.h>
#include <cstdint>

using namespace winnet::routing;

namespace
{

SOCKADDR_INET IpToNative(const WINNET_IP &from)
{
	SOCKADDR_INET to = { 0 };

	switch (from.family)
	{
		case WINNET_ADDR_FAMILY_IPV4:
		{
			to.Ipv4.sin_family = AF_INET;
			to.Ipv4.sin_addr.s_addr = *reinterpret_cast<const uint32_t*>(from.bytes);

			break;
		}
		case WINNET_ADDR_FAMILY_IPV6:
		{
			to.Ipv6.sin6_family = AF_INET6;
			memcpy(to.Ipv6.sin6_addr.u.Byte, from.bytes, 16);

			break;
		}
		default:
		{
			THROW_ERROR("Invalid network address family");
		}
	}

	return to;
}

WINNET_IP IpFromNative(const SOCKADDR_INET &from)
{
	WINNET_IP to = { 0 };

	switch (from.si_family)
	{
		case AF_INET:
		{
			*reinterpret_cast<uint32_t*>(to.bytes) = static_cast<uint32_t>(from.Ipv4.sin_addr.s_addr);
			to.family = WINNET_ADDR_FAMILY_IPV4;
			break;
		}
		case AF_INET6:
		{
			memcpy(to.bytes, from.Ipv6.sin6_addr.u.Byte, 16);
			to.family = WINNET_ADDR_FAMILY_IPV6;
			break;
		}
		default:
		{
			THROW_ERROR("Invalid network address family");
		}
	}

	return to;
}

} // anonymous namespace

namespace winnet
{

Network ConvertNetwork(const WINNET_IP_NETWORK &in)
{
	//
	// Convert WINNET_IPNETWORK into Network aka IP_ADDRESS_PREFIX
	//

	Network out = { 0 };

	out.PrefixLength = in.prefix;
	out.Prefix = IpToNative(in.addr);

	return out;
}

std::optional<Node> ConvertNode(const WINNET_NODE *in)
{
	if (nullptr == in)
	{
		return std::nullopt;
	}

	if (nullptr == in->deviceName && nullptr == in->gateway)
	{
		THROW_ERROR("Invalid 'WINNET_NODE' definition");
	}

	std::optional<std::wstring> deviceName;
	std::optional<NodeAddress> gateway;

	if (nullptr != in->deviceName)
	{
		deviceName = in->deviceName;
	}

	if (nullptr != in->gateway)
	{
		gateway = IpToNative(*in->gateway);
	}

	return Node(deviceName, gateway);
}

std::vector<Route> ConvertRoutes(const WINNET_ROUTE *routes, uint32_t numRoutes)
{
	std::vector<Route> out;

	out.reserve(numRoutes);

	for (size_t i = 0; i < numRoutes; ++i)
	{
		out.emplace_back(Route
		{
			ConvertNetwork(routes[i].network),
			ConvertNode(routes[i].node)
		});
	}

	return out;
}

std::vector<WINNET_IP> ConvertNativeAddresses(const SOCKADDR_INET *addresses, uint32_t numAddresses)
{
	std::vector<WINNET_IP> out;
	out.reserve(numAddresses);

	for (uint32_t i = 0; i < numAddresses; ++i)
	{
		out.emplace_back(IpFromNative(addresses[i]));
	}

	return out;
}

}
