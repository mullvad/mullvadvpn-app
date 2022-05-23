#include "stdafx.h"
#include "permitvpntunnel.h"
#include <winfw/mullvadguids.h>
#include <winfw/rules/shared.h>
#include <libwfp/filterbuilder.h>
#include <libwfp/conditionbuilder.h>
#include <libwfp/conditions/conditioninterface.h>
#include <libwfp/conditions/conditionip.h>
#include <libwfp/conditions/conditionport.h>
#include <libwfp/conditions/conditionprotocol.h>
#include <libcommon/error.h>

using namespace wfp::conditions;

namespace rules::baseline
{

PermitVpnTunnel::PermitVpnTunnel(
	const std::wstring &tunnelInterfaceAlias,
	const std::optional<Endpoint> &onlyEndpoint
)
	: m_tunnelInterfaceAlias(tunnelInterfaceAlias)
	, m_tunnelOnlyEndpoint(onlyEndpoint)
{
}

bool PermitVpnTunnel::apply(IObjectInstaller &objectInstaller)
{
	wfp::FilterBuilder filterBuilder;

	bool includeV4 = !m_tunnelOnlyEndpoint.has_value() || m_tunnelOnlyEndpoint->ip.type() == wfp::IpAddress::Ipv4;
	bool includeV6 = !m_tunnelOnlyEndpoint.has_value() || m_tunnelOnlyEndpoint->ip.type() == wfp::IpAddress::Ipv6;

	//
	// #1 Permit outbound connections, IPv4.
	//

	filterBuilder
		.key(MullvadGuids::Filter_Baseline_PermitVpnTunnel_Outbound_Ipv4())
		.name(L"Permit outbound connections on tunnel interface (IPv4)")
		.description(L"This filter is part of a rule that permits communications inside the VPN tunnel")
		.provider(MullvadGuids::Provider())
		.layer(FWPM_LAYER_ALE_AUTH_CONNECT_V4)
		.sublayer(MullvadGuids::SublayerBaseline())
		.weight(wfp::FilterBuilder::WeightClass::Medium)
		.permit();

	if (includeV4)
	{
		wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_CONNECT_V4);

		conditionBuilder.add_condition(ConditionInterface::Alias(m_tunnelInterfaceAlias));

		if (m_tunnelOnlyEndpoint.has_value())
		{
			conditionBuilder.add_condition(ConditionIp::Remote(m_tunnelOnlyEndpoint->ip));
			if (ProtocolHasPort(m_tunnelOnlyEndpoint->protocol))
			{
				conditionBuilder.add_condition(ConditionPort::Remote(m_tunnelOnlyEndpoint->port));
			}
			conditionBuilder.add_condition(CreateProtocolCondition(m_tunnelOnlyEndpoint->protocol));
		}

		if (!objectInstaller.addFilter(filterBuilder, conditionBuilder))
		{
			return false;
		}
	}

	//
	// #2 Permit outbound connections, IPv6.
	//

	filterBuilder
		.key(MullvadGuids::Filter_Baseline_PermitVpnTunnel_Outbound_Ipv6())
		.name(L"Permit outbound connections on tunnel interface (IPv6)")
		.layer(FWPM_LAYER_ALE_AUTH_CONNECT_V6);

	if (includeV6)
	{
		wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_CONNECT_V6);

		conditionBuilder.add_condition(ConditionInterface::Alias(m_tunnelInterfaceAlias));

		if (m_tunnelOnlyEndpoint.has_value())
		{
			conditionBuilder.add_condition(ConditionIp::Remote(m_tunnelOnlyEndpoint->ip));
			if (ProtocolHasPort(m_tunnelOnlyEndpoint->protocol))
			{
				conditionBuilder.add_condition(ConditionPort::Remote(m_tunnelOnlyEndpoint->port));
			}
			conditionBuilder.add_condition(CreateProtocolCondition(m_tunnelOnlyEndpoint->protocol));
		}

		return objectInstaller.addFilter(filterBuilder, conditionBuilder);
	}

	return true;
}

}
