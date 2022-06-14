#pragma once

#include <vector>
#include <memory>
#include <winfw/winfw.h>
#include <libwfp/conditions/conditionprotocol.h>
#include <libwfp/ipaddress.h>

namespace rules
{

using IpSet = std::vector<wfp::IpAddress>;

void SplitAddresses(const IpSet &in, IpSet &outIpv4, IpSet &outIpv6);

std::unique_ptr<wfp::conditions::ConditionProtocol> CreateProtocolCondition(WinFwProtocol protocol);

bool ProtocolHasPort(WinFwProtocol protocol);

}
