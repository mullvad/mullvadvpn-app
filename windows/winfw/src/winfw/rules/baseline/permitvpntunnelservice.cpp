#include "stdafx.h"
#include "permitvpntunnelservice.h"
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

PermitVpnTunnelService::PermitVpnTunnelService(
	const std::wstring &tunnelInterfaceAlias,
	const std::optional<PermitVpnTunnel::Endpoint> &onlyEndpoint
)
	: m_tunnelInterfaceAlias(tunnelInterfaceAlias)
	, m_tunnelOnlyEndpoint(onlyEndpoint)
{
}

bool PermitVpnTunnelService::apply(IObjectInstaller &objectInstaller)
{
	wfp::FilterBuilder filterBuilder;

	bool includeV4 = !m_tunnelOnlyEndpoint.has_value() || m_tunnelOnlyEndpoint->ip.type() == wfp::IpAddress::Ipv4;
	bool includeV6 = !m_tunnelOnlyEndpoint.has_value() || m_tunnelOnlyEndpoint->ip.type() == wfp::IpAddress::Ipv6;

	//
	// #1 Permit inbound connections, IPv4.
	//

	filterBuilder
		.key(MullvadGuids::Filter_Baseline_PermitVpnTunnelService_Ipv4())
		.name(L"Permit inbound connections on tunnel interface (IPv4)")
		.description(L"This filter is part of a rule that permits hosting services that listen on the tunnel interface")
		.provider(MullvadGuids::Provider())
		.layer(FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V4)
		.sublayer(MullvadGuids::SublayerBaseline())
		.weight(wfp::FilterBuilder::WeightClass::Medium)
		.permit();

	wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V4);

	if (includeV4)
	{
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
	// #2 Permit inbound connections, IPv6.
	//

	if (includeV6)
	{
		filterBuilder
			.key(MullvadGuids::Filter_Baseline_PermitVpnTunnelService_Ipv6())
			.name(L"Permit inbound connections on tunnel interface (IPv6)")
			.layer(FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V6);

		conditionBuilder.reset(FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V6);
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
