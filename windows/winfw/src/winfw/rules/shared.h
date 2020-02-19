#pragma once

#include <vector>
#include <libwfp/ipaddress.h>

namespace rules
{

using IpSet = std::vector<wfp::IpAddress>;

void SplitAddresses(const IpSet &in, IpSet &outIpv4, IpSet &outIpv6);

}
