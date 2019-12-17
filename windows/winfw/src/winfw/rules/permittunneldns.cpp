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
	const wfp::IpAddress v4DnsHost,
	const std::optional<wfp::IpAddress> v6DnsHost
)
	: m_tunnelInterfaceAlias(tunnelInterfaceAlias)
	, m_v4DnsHost(v4DnsHost)
	, m_v6DnsHost(v6DnsHost)
{

}

bool PermitTunnelDns::apply(IObjectInstaller &objectInstaller)
{
	//
	// Permit outbound DNS traffic to a specific DNS server (IPv4)
	//

	wfp::FilterBuilder filterBuilder;

	filterBuilder
		.provider(MullvadGuids::Provider())
		.key(MullvadGuids::FilterPermitTunnelDns_Ipv4())
		.name(L"Permit outbound DNS traffic on tunnel interface (IPv4)")
		.description(L"This filter is part of a rule that permits DNS traffic inside the VPN tunnel")
		.layer(FWPM_LAYER_ALE_AUTH_CONNECT_V4)
		.sublayer(MullvadGuids::SublayerWhitelist())
		.weight(wfp::FilterBuilder::WeightClass::Max)
		.permit();

	{
		wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_CONNECT_V4);
		conditionBuilder.add_condition(ConditionInterface::Alias(m_tunnelInterfaceAlias));
		conditionBuilder.add_condition(ConditionIp::Remote(m_v4DnsHost));
		conditionBuilder.add_condition(ConditionPort::Remote(DNS_PORT));

		if (!objectInstaller.addFilter(filterBuilder, conditionBuilder))
		{
			return false;
		}
	}

	//
	// Permit outbound DNS traffic to a specific DNS server (IPv6)
	//

	if (m_v6DnsHost.has_value())
	{
		filterBuilder
			.key(MullvadGuids::FilterPermitTunnelDns_Ipv6())
			.name(L"Permit outbound DNS traffic on tunnel interface (IPv6)")
			.layer(FWPM_LAYER_ALE_AUTH_CONNECT_V6);

		{
			wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_CONNECT_V6);

			conditionBuilder.add_condition(ConditionInterface::Alias(m_tunnelInterfaceAlias));
			conditionBuilder.add_condition(ConditionIp::Remote(m_v6DnsHost.value()));
			conditionBuilder.add_condition(ConditionPort::Remote(DNS_PORT));

			if (!objectInstaller.addFilter(filterBuilder, conditionBuilder))
			{
				return false;
			}
		}
	}

	return true;
}

}
