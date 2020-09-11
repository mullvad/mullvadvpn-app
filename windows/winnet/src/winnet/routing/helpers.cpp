#include "stdafx.h"
#include "helpers.h"
#include <ws2def.h>
#include <in6addr.h>
#include <numeric>
#include <libcommon/error.h>
#include <libcommon/memory.h>

namespace
{

bool IsRouteOnPhysicalInterface(const MIB_IPFORWARD_ROW2 &route)
{
	switch (route.InterfaceLuid.Info.IfType)
	{
		case IF_TYPE_SOFTWARE_LOOPBACK:
		case IF_TYPE_TUNNEL:
		case IF_TYPE_PROP_VIRTUAL:
		{
			return false;
		}
	}
	return true;
}

} // anonymous namespace

namespace winnet::routing
{

bool EqualAddress(const Network &lhs, const Network &rhs)
{
	if (lhs.PrefixLength != rhs.PrefixLength)
	{
		return false;
	}

	return EqualAddress(lhs.Prefix, rhs.Prefix);
}

bool EqualAddress(const NodeAddress &lhs, const NodeAddress &rhs)
{
	if (lhs.si_family != rhs.si_family)
	{
		return false;
	}

	switch (lhs.si_family)
	{
		case AF_INET:
		{
			return lhs.Ipv4.sin_addr.s_addr == rhs.Ipv4.sin_addr.s_addr;
		}
		case AF_INET6:
		{
			return 0 == memcmp(&lhs.Ipv6.sin6_addr, &rhs.Ipv6.sin6_addr, sizeof(IN6_ADDR));
		}
		default:
		{
			THROW_ERROR("Invalid address family for network address");
		}
	}
}

bool EqualAddress(const SOCKADDR_INET *lhs, const SOCKET_ADDRESS *rhs)
{
	if (lhs->si_family != rhs->lpSockaddr->sa_family)
	{
		return false;
	}

	switch (lhs->si_family)
	{
		case AF_INET:
		{
			auto typedRhs = reinterpret_cast<const SOCKADDR_IN *>(rhs->lpSockaddr);
			return lhs->Ipv4.sin_addr.s_addr == typedRhs->sin_addr.s_addr;
		}
		case AF_INET6:
		{
			auto typedRhs = reinterpret_cast<const SOCKADDR_IN6 *>(rhs->lpSockaddr);
			return 0 == memcmp(lhs->Ipv6.sin6_addr.u.Byte, typedRhs->sin6_addr.u.Byte, 16);
		}
		default:
		{
			THROW_ERROR("Missing case handler in switch clause");
		}
	}
}

bool GetAdapterInterface(NET_LUID adapter, ADDRESS_FAMILY addressFamily, MIB_IPINTERFACE_ROW *iface)
{
	memset(iface, 0, sizeof(MIB_IPINTERFACE_ROW));

	iface->Family = addressFamily;
	iface->InterfaceLuid = adapter;

	return NO_ERROR == GetIpInterfaceEntry(iface);
}

std::vector<AnnotatedRoute> AnnotateRoutes(const std::vector<const MIB_IPFORWARD_ROW2 *> &routes)
{
	std::vector<AnnotatedRoute> annotated;
	annotated.reserve(routes.size());

	for (auto route : routes)
	{
		MIB_IPINTERFACE_ROW iface;

		if (false == GetAdapterInterface(route->InterfaceLuid, route->DestinationPrefix.Prefix.si_family, &iface))
		{
			continue;
		}

		annotated.emplace_back
		(
			AnnotatedRoute{ route, bool_cast(iface.Connected), route->Metric + iface.Metric }
		);
	}

	return annotated;
}

bool RouteHasGateway(const MIB_IPFORWARD_ROW2 &route)
{
	switch (route.NextHop.si_family)
	{
		case AF_INET:
		{
			return 0 != route.NextHop.Ipv4.sin_addr.s_addr;
		}
		case AF_INET6:
		{
			const uint8_t *begin = &route.NextHop.Ipv6.sin6_addr.u.Byte[0];
			const uint8_t *end = begin + 16;

			return 0 != std::accumulate(begin, end, 0);
		}
		default:
		{
			return false;
		}
	};
}

std::optional<InterfaceAndGateway> GetBestDefaultRoute(ADDRESS_FAMILY family)
{
	PMIB_IPFORWARD_TABLE2 table;

	auto status = GetIpForwardTable2(family, &table);

	if (NO_ERROR != status)
	{
		THROW_WINDOWS_ERROR(status, "Acquire route table");
	}

	common::memory::ScopeDestructor sd;

	sd += [table]
	{
		FreeMibTable(table);
	};

	std::vector<const MIB_IPFORWARD_ROW2 *> candidates;
	candidates.reserve(table->NumEntries);

	//
	// Enumerate routes looking for: route 0/0
	// The WireGuard interface route has no gateway.
	//

	for (ULONG i = 0; i < table->NumEntries; ++i)
	{
		const MIB_IPFORWARD_ROW2 &candidate = table->Table[i];

		if (0 == candidate.DestinationPrefix.PrefixLength
			&& RouteHasGateway(candidate)
			&& IsRouteOnPhysicalInterface(candidate))
		{
			candidates.emplace_back(&candidate);
		}
	}

	auto annotated = AnnotateRoutes(candidates);

	if (annotated.empty())
	{
		return std::nullopt;
	}

	//
	// Sort on (active, effectiveMetric) ascending by metric.
	//

	std::sort(annotated.begin(), annotated.end(), [](const AnnotatedRoute &lhs, const AnnotatedRoute &rhs)
	{
		if (lhs.active == rhs.active)
		{
			return lhs.effectiveMetric < rhs.effectiveMetric;
		}

		return lhs.active && false == rhs.active;
	});

	//
	// Ensure the top rated route is active.
	//

	if (false == annotated[0].active)
	{
		return std::nullopt;
	}

	return std::make_optional(InterfaceAndGateway { annotated[0].route->InterfaceLuid, annotated[0].route->NextHop });
}

bool AdapterInterfaceEnabled(const IP_ADAPTER_ADDRESSES *adapter, ADDRESS_FAMILY family)
{
	switch (family)
	{
		case AF_INET:
		{
			return 0 != adapter->Ipv4Enabled;
		}
		case AF_INET6:
		{
			return 0 != adapter->Ipv6Enabled;
		}
		default:
		{
			THROW_ERROR("Missing case handler in switch clause");
		}
	}
}

std::vector<const SOCKET_ADDRESS *> IsolateGatewayAddresses
(
	PIP_ADAPTER_GATEWAY_ADDRESS_LH head,
	ADDRESS_FAMILY family
)
{
	std::vector<const SOCKET_ADDRESS *> matches;

	for (auto gateway = head; nullptr != gateway; gateway = gateway->Next)
	{
		if (family == gateway->Address.lpSockaddr->sa_family)
		{
			matches.emplace_back(&gateway->Address);
		}
	}

	return matches;
}

bool AddressPresent(const std::vector<const SOCKET_ADDRESS *> &hay, const SOCKADDR_INET *needle)
{
	for (const auto candidate : hay)
	{
		if (EqualAddress(needle, candidate))
		{
			return true;
		}
	}

	return false;
}

}
