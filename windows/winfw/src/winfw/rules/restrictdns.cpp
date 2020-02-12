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
	const std::optional<WinFwRelay> &relay,
	const std::optional<DnsHosts> &dnsHosts
)
	: m_dnsHosts(dnsHosts)
{
	if (relay.has_value() && 53 == relay->port)
	{
		m_allowHost = std::make_optional(wfp::IpAddress(relay->ip));
	}
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
		.sublayer(MullvadGuids::SublayerBlacklist());

	if (m_dnsHosts.has_value())
	{
		filterBuilder
			.key(MullvadGuids::FilterRestrictDns_Outbound_Tunnel_Ipv4())
			.name(L"Restrict DNS requests inside the VPN tunnel (IPv4)")
			.layer(FWPM_LAYER_ALE_AUTH_CONNECT_V4)
			.weight(MAXUINT16)
			.permit();

		{
			wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_CONNECT_V4);

			conditionBuilder.add_condition(ConditionInterface::Alias(m_dnsHosts->tunnelInterfaceAlias, CompareEq()));
			conditionBuilder.add_condition(ConditionIp::Remote(m_dnsHosts->v4DnsHost, CompareEq()));

			if (!objectInstaller.addFilter(filterBuilder, conditionBuilder))
			{
				return false;
			}
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

		//
		// Allow DNS traffic over select host
		//
		if (m_allowHost.has_value())
		{
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

	if (m_dnsHosts.has_value() && m_dnsHosts->v6DnsHost.has_value())
	{
		filterBuilder
			.key(MullvadGuids::FilterRestrictDns_Outbound_Tunnel_Ipv6())
			.name(L"Restrict DNS requests inside the VPN tunnel (IPv6)")
			.layer(FWPM_LAYER_ALE_AUTH_CONNECT_V6)
			.weight(MAXUINT16)
			.permit();

		{
			wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_CONNECT_V6);

			conditionBuilder.add_condition(ConditionInterface::Alias(m_dnsHosts->tunnelInterfaceAlias, CompareEq()));
			conditionBuilder.add_condition(ConditionIp::Remote(*m_dnsHosts->v6DnsHost, CompareEq()));

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
