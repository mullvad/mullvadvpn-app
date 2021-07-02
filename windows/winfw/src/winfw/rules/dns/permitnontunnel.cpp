#include "stdafx.h"
#include "permitnontunnel.h"
#include <winfw/mullvadguids.h>
#include <winfw/rules/ports.h>
#include <winfw/rules/shared.h>
#include <libwfp/filterbuilder.h>
#include <libwfp/conditionbuilder.h>
#include <libwfp/conditions/conditionport.h>
#include <libwfp/conditions/conditionip.h>
#include <libwfp/conditions/conditioninterface.h>
#include <libcommon/error.h>

using namespace wfp::conditions;

namespace rules::dns
{

PermitNonTunnel::PermitNonTunnel(std::optional<std::wstring> tunnelInterfaceAlias, const std::vector<wfp::IpAddress> &hosts)
	: m_tunnelInterfaceAlias(std::move(tunnelInterfaceAlias))
{
	SplitAddresses(hosts, m_hostsIpv4, m_hostsIpv6);
}

bool PermitNonTunnel::apply(IObjectInstaller &objectInstaller)
{
	wfp::FilterBuilder filterBuilder;

	//
	// #1 Permit outbound DNS, IPv4.
	//

	if (false == m_hostsIpv4.empty())
	{
		filterBuilder
			.key(MullvadGuids::Filter_Dns_PermitNonTunnel_Outbound_Ipv4())
			.name(L"Permit selected non-tunnel DNS traffic (IPv4)")
			.description(L"This filter is part of a rule that permits non-tunnel DNS traffic")
			.provider(MullvadGuids::Provider())
			.layer(FWPM_LAYER_ALE_AUTH_CONNECT_V4)
			.sublayer(MullvadGuids::SublayerDns())
			.weight(wfp::FilterBuilder::WeightClass::Medium)
			.permit();

		wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_CONNECT_V4);

		conditionBuilder.add_condition(ConditionPort::Remote(DNS_SERVER_PORT));

		for (const auto &host : m_hostsIpv4)
		{
			conditionBuilder.add_condition(ConditionIp::Remote(host));
		}

		if (m_tunnelInterfaceAlias.has_value())
		{
			conditionBuilder.add_condition(ConditionInterface::Alias(m_tunnelInterfaceAlias.value(), CompareNeq()));
		}

		if (false == objectInstaller.addFilter(filterBuilder, conditionBuilder))
		{
			return false;
		}
	}

	if (m_hostsIpv6.empty())
	{
		return true;
	}

	//
	// #2 Permit outbound DNS, IPv6.
	//

	filterBuilder
		.key(MullvadGuids::Filter_Dns_PermitNonTunnel_Outbound_Ipv6())
		.name(L"Permit selected non-tunnel DNS traffic (IPv6)")
		.description(L"This filter is part of a rule that permits non-tunnel DNS traffic")
		.provider(MullvadGuids::Provider())
		.layer(FWPM_LAYER_ALE_AUTH_CONNECT_V6)
		.sublayer(MullvadGuids::SublayerDns())
		.weight(wfp::FilterBuilder::WeightClass::Medium)
		.permit();

	wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_CONNECT_V6);

	conditionBuilder.add_condition(ConditionPort::Remote(DNS_SERVER_PORT));

	for (const auto &host : m_hostsIpv6)
	{
		conditionBuilder.add_condition(ConditionIp::Remote(host));
	}

	if (m_tunnelInterfaceAlias.has_value())
	{
		conditionBuilder.add_condition(ConditionInterface::Alias(m_tunnelInterfaceAlias.value(), CompareNeq()));
	}

	return objectInstaller.addFilter(filterBuilder, conditionBuilder);
}

}
