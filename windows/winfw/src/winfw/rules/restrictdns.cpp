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

RestrictDns::RestrictDns(const std::wstring &tunnelInterfaceAlias, const wfp::IpAddress v4DnsHost, std::unique_ptr<wfp::IpAddress> v6DnsHost)
	: m_tunnelInterfaceAlias(tunnelInterfaceAlias)
	, m_v4DnsHost(v4DnsHost)
	, m_v6DnsHost(std::move(v6DnsHost))

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
		.name(L"Block DNS requests outside the VPN tunnel (IPv4)")
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

	filterBuilder
		.name(L"Restrict DNS requests inside the VPN tunnel (IPv4)")
		.key(MullvadGuids::FilterRestrictDns_Outbound_Tunnel_Ipv4())
		.layer(FWPM_LAYER_ALE_AUTH_CONNECT_V4);

	{
		wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_CONNECT_V4);

		conditionBuilder.add_condition(ConditionPort::Remote(53));
		conditionBuilder.add_condition(ConditionIp::Remote(m_v4DnsHost, CompareNeq()));

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
		.name(L"Block DNS requests outside the VPN tunnel (IPv6)")
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

	filterBuilder
		.key(MullvadGuids::FilterRestrictDns_Outbound_Tunnel_Ipv6())
		.name(L"Restrict DNS requests inside the VPN tunnel (IPv6)")
		.layer(FWPM_LAYER_ALE_AUTH_CONNECT_V6);

	{
		wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_CONNECT_V6);

		conditionBuilder.add_condition(ConditionPort::Remote(53));

		if (m_v6DnsHost != nullptr)
		{
			conditionBuilder.add_condition(ConditionIp::Remote(*m_v6DnsHost, CompareNeq()));
		}

		return objectInstaller.addFilter(filterBuilder, conditionBuilder);
	}
}

}
