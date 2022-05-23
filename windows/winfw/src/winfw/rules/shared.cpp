#include "stdafx.h"
#include "shared.h"
#include <libcommon/error.h>

namespace rules
{

void SplitAddresses(const IpSet &in, IpSet &outIpv4, IpSet &outIpv6)
{
	if (in.empty())
	{
		THROW_ERROR("Invalid argument: No hosts specified");
	}

	outIpv4.clear();
	outIpv6.clear();

	for (const auto &host : in)
	{
		switch (host.type())
		{
			case wfp::IpAddress::Type::Ipv4:
			{
				outIpv4.push_back(host);
				break;
			}
			case wfp::IpAddress::Type::Ipv6:
			{
				outIpv6.push_back(host);
				break;
			}
			default:
			{
				THROW_ERROR("Missing case handler in switch clause");
			}
		}
	}
}

std::unique_ptr<wfp::conditions::ConditionProtocol> CreateProtocolCondition(WinFwProtocol protocol)
{
	switch (protocol)
	{
		case WinFwProtocol::Tcp: return ConditionProtocol::Tcp();
		case WinFwProtocol::Udp: return ConditionProtocol::Udp();
		case WinFwProtocol::Icmp: return ConditionProtocol::Icmp();
		case WinFwProtocol::IcmpV6: return ConditionProtocol::IcmpV6();
		default:
		{
			THROW_ERROR("Missing case handler in switch clause");
		}
	};
}

bool ProtocolHasPort(WinFwProtocol protocol)
{
	switch (protocol)
	{
		case WinFwProtocol::Tcp:
		case WinFwProtocol::Udp:
			return true;
		case WinFwProtocol::Icmp:
		case WinFwProtocol::IcmpV6:
			return false;
		default:
		{
			THROW_ERROR("Missing case handler in switch clause");
		}
	};
}

}
