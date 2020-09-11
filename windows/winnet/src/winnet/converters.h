#pragma once

#include "winnet.h"
#include "routing/types.h"
#include <optional>
#include <vector>

namespace winnet
{

routing::Network ConvertNetwork(const WINNET_IP_NETWORK &in);
std::optional<routing::Node> ConvertNode(const WINNET_NODE *in);
std::vector<routing::Route> ConvertRoutes(const WINNET_ROUTE *routes, uint32_t numRoutes);
std::vector<SOCKADDR_INET> ConvertAddresses(const WINNET_IP *addresses, uint32_t numAddresses);
std::vector<WINNET_IP> ConvertNativeAddresses(const SOCKADDR_INET *addresses, uint32_t numAddresses);

}
