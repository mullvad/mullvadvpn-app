#pragma once

#include "types.h"
#include <vector>

namespace winnet::routing
{

bool EqualAddress(const Network &lhs, const Network &rhs);
bool EqualAddress(const NodeAddress &lhs, const NodeAddress &rhs);
bool EqualAddress(const SOCKADDR_INET *lhs, const SOCKET_ADDRESS *rhs);

bool GetAdapterInterface(NET_LUID adapter, ADDRESS_FAMILY addressFamily, MIB_IPINTERFACE_ROW *iface);

struct AnnotatedRoute
{
	const MIB_IPFORWARD_ROW2 *route;
	bool active;
	uint32_t effectiveMetric;
};

template<typename T>
bool bool_cast(const T &value)
{
	return 0 != value;
}

std::vector<AnnotatedRoute> AnnotateRoutes(const std::vector<const MIB_IPFORWARD_ROW2 *> &routes);

bool RouteHasGateway(const MIB_IPFORWARD_ROW2 &route);

InterfaceAndGateway GetBestDefaultRoute(ADDRESS_FAMILY family);

bool AdapterInterfaceEnabled(const IP_ADAPTER_ADDRESSES *adapter, ADDRESS_FAMILY family);

std::vector<const SOCKET_ADDRESS *> IsolateGatewayAddresses
(
	PIP_ADAPTER_GATEWAY_ADDRESS_LH head,
	ADDRESS_FAMILY family
);

bool AddressPresent(const std::vector<const SOCKET_ADDRESS *> &hay, const SOCKADDR_INET *needle);

//NodeAddress ConvertSocketAddress(const SOCKET_ADDRESS *sa);

}
