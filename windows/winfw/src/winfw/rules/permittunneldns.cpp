#include "stdafx.h"
#include "permittunneldns.h"
#include "winfw/mullvadguids.h"
#include "libwfp/filterbuilder.h"
#include "libwfp/conditionbuilder.h"
#include "libwfp/conditions/comparison.h"
#include "libwfp/conditions/conditioninterface.h"
#include "libwfp/conditions/conditionip.h"
#include "libwfp/conditions/conditionport.h"

using namespace wfp::conditions;

namespace
{

constexpr uint16_t DNS_PORT = 53;

} // anonymous namespace

namespace rules
{

PermitTunnelDns::PermitTunnelDns(
	const std::wstring &tunnelInterfaceAlias,
	const std::vector<wfp::IpAddress> &dnsHosts
)
	: m_tunnelInterfaceAlias(tunnelInterfaceAlias)
{
	for (const auto &host : dnsHosts)
	{
		if (wfp::IpAddress::Ipv4 == host.type())
		{
			m_v4DnsHosts.push_back(host);
		}
		else
		{
			m_v6DnsHosts.push_back(host);
		}
	}
}

bool PermitTunnelDns::apply(IObjectInstaller &objectInstaller)
{
	//
	// Permit outbound DNS traffic to specific servers (IPv4)
	//

	wfp::FilterBuilder filterBuilder;

	filterBuilder
		.provider(MullvadGuids::Provider())
		.description(L"This filter is part of a rule that permits DNS traffic inside the VPN tunnel")
		.sublayer(MullvadGuids::SublayerWhitelist())
		.weight(wfp::FilterBuilder::WeightClass::Max)
		.permit();

	if (!m_v4DnsHosts.empty())
	{
		filterBuilder
			.key(MullvadGuids::FilterPermitTunnelDns_Ipv4())
			.name(L"Permit select outbound DNS traffic on tunnel interface (IPv4)")
			.layer(FWPM_LAYER_ALE_AUTH_CONNECT_V4);

		wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_CONNECT_V4);
		conditionBuilder.add_condition(ConditionInterface::Alias(m_tunnelInterfaceAlias));

		for (const auto &host : m_v4DnsHosts)
		{
			// Multiple conditions of same type are OR'ed
			conditionBuilder.add_condition(ConditionIp::Remote(host));
		}

		conditionBuilder.add_condition(ConditionPort::Remote(DNS_PORT));

		if (!objectInstaller.addFilter(filterBuilder, conditionBuilder))
		{
			return false;
		}
	}

	//
	// Permit outbound DNS traffic to specific servers (IPv6)
	//

	if (!m_v6DnsHosts.empty())
	{
		filterBuilder
			.key(MullvadGuids::FilterPermitTunnelDns_Ipv6())
			.name(L"Permit select outbound DNS traffic on tunnel interface (IPv6)")
			.layer(FWPM_LAYER_ALE_AUTH_CONNECT_V6);

		wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_CONNECT_V6);
		conditionBuilder.add_condition(ConditionInterface::Alias(m_tunnelInterfaceAlias));

		for (const auto &host : m_v6DnsHosts)
		{
			// Multiple conditions of same type are OR'ed
			if (wfp::IpAddress::Ipv6 == host.type())
			{
				conditionBuilder.add_condition(ConditionIp::Remote(host));
			}
		}

		conditionBuilder.add_condition(ConditionPort::Remote(DNS_PORT));

		if (!objectInstaller.addFilter(filterBuilder, conditionBuilder))
		{
			return false;
		}
	}

	return true;
}

}
