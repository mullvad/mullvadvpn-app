#include "stdafx.h"
#include "restrictdns.h"
#include "wfpctl/mullvadguids.h"
#include "libwfp/filterbuilder.h"
#include "libwfp/conditionbuilder.h"
#include "libwfp/conditions/conditioninterface.h"
#include "libwfp/conditions/conditionip.h"
#include "libwfp/conditions/conditionport.h"

using namespace wfp::conditions;

namespace rules
{

RestrictDns::RestrictDns(const std::wstring &tunnelInterfaceAlias, const wfp::IpAddress &dns)
	: m_tunnelInterfaceAlias(tunnelInterfaceAlias)
	, m_dns(dns)
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
		.key(MullvadGuids::FilterRestrictDns_Outbound_Ipv4())
		.name(L"Block DNS requests outside the VPN tunnel")
		.description(L"This filter is part of a rule that restricts DNS traffic")
		.provider(MullvadGuids::Provider())
		.layer(FWPM_LAYER_ALE_AUTH_CONNECT_V4)
		.sublayer(MullvadGuids::SublayerBlacklist())
		.weight(wfp::FilterBuilder::WeightClass::Max)
		.block();

	{
		wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_CONNECT_V4);

		conditionBuilder.add_condition(ConditionPort::Remote(53));
		conditionBuilder.add_condition(ConditionInterface::Alias(m_tunnelInterfaceAlias, CompareNeq()));

		if (!objectInstaller.addFilter(filterBuilder, conditionBuilder))
		{
			return false;
		}
	}

	//
	// IPv6 also
	//

	filterBuilder
		.key(MullvadGuids::FilterRestrictDns_Outbound_Ipv6())
		.layer(FWPM_LAYER_ALE_AUTH_CONNECT_V6);

	{
		wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_CONNECT_V6);

		conditionBuilder.add_condition(ConditionPort::Remote(53));
		conditionBuilder.add_condition(ConditionInterface::Alias(m_tunnelInterfaceAlias, CompareNeq()));

		if (!objectInstaller.addFilter(filterBuilder, conditionBuilder))
		{
			return false;
		}
	}

	//
	// This next part is a little redundant since the entire rule could be defined
	// using three filters. Let's use four filters to maintain some kind of readability.
	//
	// The reason it would be possible to use three filters is because the single DNS
	// is going to be either v4 or v6, so all requests that cannot be sent to the DNS
	// will have to be blocked (thereby shadowing one of the filters above).
	//

	filterBuilder
		.name(L"Restrict DNS requests inside the VPN tunnel");

	if (m_dns.type() == wfp::IpAddress::Type::Ipv4)
	{
		filterBuilder
			.key(MullvadGuids::FilterRestrictDns_Outbound_Tunnel_Ipv4())
			.layer(FWPM_LAYER_ALE_AUTH_CONNECT_V4);

		{
			wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_CONNECT_V4);

			conditionBuilder.add_condition(ConditionPort::Remote(53));
			conditionBuilder.add_condition(ConditionIp::Remote(m_dns, CompareNeq()));

			if (!objectInstaller.addFilter(filterBuilder, conditionBuilder))
			{
				return false;
			}
		}

		filterBuilder
			.key(MullvadGuids::FilterRestrictDns_Outbound_Tunnel_Ipv6())
			.layer(FWPM_LAYER_ALE_AUTH_CONNECT_V6);

		wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_CONNECT_V6);

		conditionBuilder.add_condition(ConditionPort::Remote(53));

		return objectInstaller.addFilter(filterBuilder, conditionBuilder);
	}

	//
	// Specified DNS is IPv6
	//

	filterBuilder
		.key(MullvadGuids::FilterRestrictDns_Outbound_Tunnel_Ipv6())
		.layer(FWPM_LAYER_ALE_AUTH_CONNECT_V6);

	{
		wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_CONNECT_V6);

		conditionBuilder.add_condition(ConditionPort::Remote(53));
		conditionBuilder.add_condition(ConditionIp::Remote(m_dns, CompareNeq()));

		if (!objectInstaller.addFilter(filterBuilder, conditionBuilder))
		{
			return false;
		}
	}

	filterBuilder
		.key(MullvadGuids::FilterRestrictDns_Outbound_Tunnel_Ipv4())
		.layer(FWPM_LAYER_ALE_AUTH_CONNECT_V4);

	wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_CONNECT_V4);

	conditionBuilder.add_condition(ConditionPort::Remote(53));

	return objectInstaller.addFilter(filterBuilder, conditionBuilder);
}

}
