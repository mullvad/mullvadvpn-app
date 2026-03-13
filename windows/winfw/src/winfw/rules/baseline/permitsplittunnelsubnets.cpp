#include "stdafx.h"
#include "permitsplittunnelsubnets.h"
#include <winfw/mullvadguids.h>
#include <libwfp/filterbuilder.h>
#include <libwfp/conditionbuilder.h>
#include <libwfp/conditions/conditionip.h>

using namespace wfp::conditions;

namespace rules::baseline
{

PermitSplitTunnelSubnets::PermitSplitTunnelSubnets(
	const std::vector<wfp::IpNetwork> &ipv4Subnets,
	const std::vector<wfp::IpNetwork> &ipv6Subnets
)
	: m_ipv4Subnets(ipv4Subnets)
	, m_ipv6Subnets(ipv6Subnets)
{
}

bool PermitSplitTunnelSubnets::apply(IObjectInstaller &objectInstaller)
{
	if (!m_ipv4Subnets.empty())
	{
		if (!applyIpv4Outbound(objectInstaller) || !applyIpv4Inbound(objectInstaller))
		{
			return false;
		}
	}

	if (!m_ipv6Subnets.empty())
	{
		if (!applyIpv6Outbound(objectInstaller) || !applyIpv6Inbound(objectInstaller))
		{
			return false;
		}
	}

	return true;
}

bool PermitSplitTunnelSubnets::applyIpv4Outbound(IObjectInstaller &objectInstaller) const
{
	wfp::FilterBuilder filterBuilder;

	filterBuilder
		.key(MullvadGuids::Filter_Baseline_PermitSplitTunnelSubnets_Outbound_Ipv4())
		.name(L"Permit outbound to split tunnel subnets (IPv4)")
		.description(L"This filter permits outbound traffic to user-specified IP networks for split tunneling")
		.provider(MullvadGuids::Provider())
		.layer(FWPM_LAYER_ALE_AUTH_CONNECT_V4)
		.sublayer(MullvadGuids::SublayerBaseline())
		.weight(wfp::FilterBuilder::WeightClass::Medium)
		.permit();

	wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_CONNECT_V4);

	for (const auto &network : m_ipv4Subnets)
	{
		conditionBuilder.add_condition(ConditionIp::Remote(network));
	}

	return objectInstaller.addFilter(filterBuilder, conditionBuilder);
}

bool PermitSplitTunnelSubnets::applyIpv4Inbound(IObjectInstaller &objectInstaller) const
{
	wfp::FilterBuilder filterBuilder;

	filterBuilder
		.key(MullvadGuids::Filter_Baseline_PermitSplitTunnelSubnets_Inbound_Ipv4())
		.name(L"Permit inbound from split tunnel subnets (IPv4)")
		.description(L"This filter permits inbound traffic from user-specified IP networks for split tunneling")
		.provider(MullvadGuids::Provider())
		.layer(FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V4)
		.sublayer(MullvadGuids::SublayerBaseline())
		.weight(wfp::FilterBuilder::WeightClass::Medium)
		.permit();

	wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V4);

	for (const auto &network : m_ipv4Subnets)
	{
		conditionBuilder.add_condition(ConditionIp::Remote(network));
	}

	return objectInstaller.addFilter(filterBuilder, conditionBuilder);
}

bool PermitSplitTunnelSubnets::applyIpv6Outbound(IObjectInstaller &objectInstaller) const
{
	wfp::FilterBuilder filterBuilder;

	filterBuilder
		.key(MullvadGuids::Filter_Baseline_PermitSplitTunnelSubnets_Outbound_Ipv6())
		.name(L"Permit outbound to split tunnel subnets (IPv6)")
		.description(L"This filter permits outbound traffic to user-specified IP networks for split tunneling")
		.provider(MullvadGuids::Provider())
		.layer(FWPM_LAYER_ALE_AUTH_CONNECT_V6)
		.sublayer(MullvadGuids::SublayerBaseline())
		.weight(wfp::FilterBuilder::WeightClass::Medium)
		.permit();

	wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_CONNECT_V6);

	for (const auto &network : m_ipv6Subnets)
	{
		conditionBuilder.add_condition(ConditionIp::Remote(network));
	}

	return objectInstaller.addFilter(filterBuilder, conditionBuilder);
}

bool PermitSplitTunnelSubnets::applyIpv6Inbound(IObjectInstaller &objectInstaller) const
{
	wfp::FilterBuilder filterBuilder;

	filterBuilder
		.key(MullvadGuids::Filter_Baseline_PermitSplitTunnelSubnets_Inbound_Ipv6())
		.name(L"Permit inbound from split tunnel subnets (IPv6)")
		.description(L"This filter permits inbound traffic from user-specified IP networks for split tunneling")
		.provider(MullvadGuids::Provider())
		.layer(FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V6)
		.sublayer(MullvadGuids::SublayerBaseline())
		.weight(wfp::FilterBuilder::WeightClass::Medium)
		.permit();

	wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V6);

	for (const auto &network : m_ipv6Subnets)
	{
		conditionBuilder.add_condition(ConditionIp::Remote(network));
	}

	return objectInstaller.addFilter(filterBuilder, conditionBuilder);
}

}
