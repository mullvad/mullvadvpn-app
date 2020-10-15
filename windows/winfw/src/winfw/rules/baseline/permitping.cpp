#include "stdafx.h"
#include "permitping.h"
#include <winfw/mullvadguids.h>
#include <winfw/rules/shared.h>
#include <libwfp/filterbuilder.h>
#include <libwfp/conditionbuilder.h>
#include <libwfp/conditions/conditionip.h>
#include <libwfp/conditions/conditioninterface.h>
#include <libwfp/conditions/conditionprotocol.h>
#include <libcommon/error.h>

using namespace wfp::conditions;

namespace rules::baseline
{

PermitPing::PermitPing
(
	std::optional<std::wstring> interfaceAlias,
	const std::vector<wfp::IpAddress> &hosts
)
	: m_interfaceAlias(std::move(interfaceAlias))
{
	SplitAddresses(hosts, m_hostsIpv4, m_hostsIpv6);
}

bool PermitPing::apply(IObjectInstaller &objectInstaller)
{
	if (false == m_hostsIpv4.empty())
	{
		if (false == applyIcmpv4(objectInstaller))
		{
			return false;
		}
	}

	if (false == m_hostsIpv6.empty())
	{
		if (false == applyIcmpv6(objectInstaller))
		{
			return false;
		}
	}

	return true;
}

bool PermitPing::applyIcmpv4(IObjectInstaller &objectInstaller) const
{
	wfp::FilterBuilder filterBuilder;

	//
	// #1 Permit outbound ICMPv4 to %host% on %interface%.
	//

	filterBuilder
		.key(MullvadGuids::Filter_Baseline_PermitPing_Outbound_Icmpv4())
		.name(L"Permit outbound ICMP to specific host (ICMPv4)")
		.description(L"This filter is part of a rule that permits ping")
		.provider(MullvadGuids::Provider())
		.layer(FWPM_LAYER_ALE_AUTH_CONNECT_V4)
		.sublayer(MullvadGuids::SublayerBaseline())
		.weight(wfp::FilterBuilder::WeightClass::Medium)
		.permit();

	wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_CONNECT_V4);

	conditionBuilder.add_condition(ConditionProtocol::Icmp());

	for (const auto &host : m_hostsIpv4)
	{
		conditionBuilder.add_condition(ConditionIp::Remote(host));
	}

	if (m_interfaceAlias.has_value())
	{
		conditionBuilder.add_condition(ConditionInterface::Alias(m_interfaceAlias.value()));
	}

	return objectInstaller.addFilter(filterBuilder, conditionBuilder);
}

bool PermitPing::applyIcmpv6(IObjectInstaller &objectInstaller) const
{
	wfp::FilterBuilder filterBuilder;

	//
	// #1 Permit outbound ICMPv6 to %host% on %interface%.
	//

	filterBuilder
		.key(MullvadGuids::Filter_Baseline_PermitPing_Outbound_Icmpv6())
		.name(L"Permit outbound ICMP to specific host (ICMPv6)")
		.description(L"This filter is part of a rule that permits ping")
		.provider(MullvadGuids::Provider())
		.layer(FWPM_LAYER_ALE_AUTH_CONNECT_V6)
		.sublayer(MullvadGuids::SublayerBaseline())
		.weight(wfp::FilterBuilder::WeightClass::Medium)
		.permit();

	wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_CONNECT_V6);

	conditionBuilder.add_condition(ConditionProtocol::IcmpV6());

	for (const auto &host : m_hostsIpv6)
	{
		conditionBuilder.add_condition(ConditionIp::Remote(host));
	}

	if (m_interfaceAlias.has_value())
	{
		conditionBuilder.add_condition(ConditionInterface::Alias(m_interfaceAlias.value()));
	}

	return objectInstaller.addFilter(filterBuilder, conditionBuilder);
}

}
