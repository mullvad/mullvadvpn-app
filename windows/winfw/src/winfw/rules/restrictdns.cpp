#include "stdafx.h"
#include "restrictdns.h"
#include "winfw/mullvadguids.h"
#include "libwfp/filterbuilder.h"
#include "libwfp/conditionbuilder.h"
#include "libwfp/conditions/conditioninterface.h"
#include "libwfp/conditions/conditionip.h"
#include "libwfp/conditions/conditionport.h"

using namespace wfp::conditions;

namespace rules
{

RestrictDns::RestrictDns(
	const std::wstring &tunnelInterfaceAlias,
	const wfp::IpAddress v4DnsHost,
	std::optional<wfp::IpAddress> v6DnsHost,
	std::optional<wfp::IpAddress> allowHost
)
	: m_tunnelInterfaceAlias(tunnelInterfaceAlias)
	, m_v4DnsHost(v4DnsHost)
	, m_v6DnsHost(v6DnsHost)
	, m_allowHost(allowHost)
{
}

bool RestrictDns::apply(IObjectInstaller &objectInstaller)
{
	wfp::FilterBuilder filterBuilder;

	//
	// Requires that the following rules are in effect:
	//
	// BlockAll
	// PermitVpnTunnel
	//
	// TODO: Have each rule specify requirements?
	//

	filterBuilder
		.provider(MullvadGuids::Provider())
		.description(L"This filter is part of a rule that restricts DNS traffic")
		.sublayer(MullvadGuids::SublayerBlacklist())
		.key(MullvadGuids::FilterRestrictDns_Outbound_Tunnel_Ipv4())
		.name(L"Restrict DNS requests inside the VPN tunnel (IPv4)")
		.layer(FWPM_LAYER_ALE_AUTH_CONNECT_V4)
		.weight(MAXUINT16)
		.permit();

	{
		wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_CONNECT_V4);

		conditionBuilder.add_condition(ConditionInterface::Alias(m_tunnelInterfaceAlias, CompareEq()));
		conditionBuilder.add_condition(ConditionIp::Remote(m_v4DnsHost, CompareEq()));

		if (!objectInstaller.addFilter(filterBuilder, conditionBuilder))
		{
			return false;
		}
	}

	filterBuilder
		.key(MullvadGuids::FilterRestrictDns_Outbound_Ipv4())
		.name(L"Block DNS requests outside the VPN tunnel (IPv4)")
		.layer(FWPM_LAYER_ALE_AUTH_CONNECT_V4)
		.weight(MAXUINT16 - 1)
		.block();

	{
		wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_CONNECT_V4);
		conditionBuilder.add_condition(ConditionPort::Remote(53));

		if (m_allowHost.has_value())
		{
			//
			// Allow DNS traffic over select host
			//
			conditionBuilder.add_condition(ConditionIp::Remote(*m_allowHost, CompareNeq()));
		}

		if (!objectInstaller.addFilter(filterBuilder, conditionBuilder))
		{
			return false;
		}
	}

	//
	// IPv6 also
	//

	if (m_v6DnsHost.has_value())
	{
		filterBuilder
			.key(MullvadGuids::FilterRestrictDns_Outbound_Tunnel_Ipv6())
			.name(L"Restrict DNS requests inside the VPN tunnel (IPv6)")
			.layer(FWPM_LAYER_ALE_AUTH_CONNECT_V6)
			.weight(MAXUINT16)
			.permit();

		{
			wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_CONNECT_V6);

			conditionBuilder.add_condition(ConditionInterface::Alias(m_tunnelInterfaceAlias, CompareEq()));
			conditionBuilder.add_condition(ConditionIp::Remote(m_v6DnsHost.value(), CompareEq()));

			if (!objectInstaller.addFilter(filterBuilder, conditionBuilder))
			{
				return false;
			}
		}
	}

	filterBuilder
		.key(MullvadGuids::FilterRestrictDns_Outbound_Ipv6())
		.name(L"Block DNS requests outside the VPN tunnel (IPv6)")
		.layer(FWPM_LAYER_ALE_AUTH_CONNECT_V6)
		.weight(MAXUINT16 - 1)
		.block();

	{
		wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_CONNECT_V6);
		conditionBuilder.add_condition(ConditionPort::Remote(53));
		return objectInstaller.addFilter(filterBuilder, conditionBuilder);
	}
}

}
