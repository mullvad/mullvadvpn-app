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
using Endpoint = PermitVpnTunnel::Endpoint;

PermitVpnTunnelService::PermitVpnTunnelService(
	const std::wstring &tunnelInterfaceAlias,
	const std::optional<PermitVpnTunnel::Endpoints> &potentialEndpoints
)
	: m_tunnelInterfaceAlias(tunnelInterfaceAlias)
	, m_potentialEndpoints(potentialEndpoints)
{
}

bool PermitVpnTunnelService::AddEndpointFilter(const std::optional<PermitVpnTunnel::Endpoint> &endpoint, const GUID &ipv4Guid, const GUID &ipv6Guid, IObjectInstaller &objectInstaller)
{
	wfp::FilterBuilder filterBuilder;

	filterBuilder
		.description(L"This filter is part of a rule that permits hosting services that listen on the tunnel interface")
		.provider(MullvadGuids::Provider())
		.layer(FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V4)
		.sublayer(MullvadGuids::SublayerBaseline())
		.weight(wfp::FilterBuilder::WeightClass::Medium)
		.permit();

    bool shouldAddV4Filter = !endpoint.has_value() || endpoint.value().ip.type() == wfp::IpAddress::Ipv4;
    bool shouldAddV6Filter = !endpoint.has_value() || endpoint.value().ip.type() == wfp::IpAddress::Ipv6;

	if (shouldAddV4Filter)
	{
		filterBuilder
			.key(ipv4Guid)
			.name(L"Permit inbound connections on tunnel interface (IPv4)")
			.layer(FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V4);

		wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_CONNECT_V4);

		conditionBuilder.add_condition(ConditionInterface::Alias(m_tunnelInterfaceAlias));
		if (endpoint.has_value())
		{
			conditionBuilder.add_condition(ConditionIp::Remote(endpoint.value().ip));
			conditionBuilder.add_condition(ConditionPort::Remote(endpoint.value().port));
			conditionBuilder.add_condition(CreateProtocolCondition(endpoint.value().protocol));
		}

		if (!objectInstaller.addFilter(filterBuilder, conditionBuilder))
		{
			return false;
		}
	}

	if (shouldAddV6Filter)
	{
		filterBuilder
			.key(ipv6Guid)
			.name(L"Permit inbound connections on tunnel interface (IPv6)")
			.layer(FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V6);

		wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_CONNECT_V6);

		conditionBuilder.add_condition(ConditionInterface::Alias(m_tunnelInterfaceAlias));
		if (endpoint.has_value())
		{
			conditionBuilder.add_condition(ConditionIp::Remote(endpoint.value().ip));
			conditionBuilder.add_condition(ConditionPort::Remote(endpoint.value().port));
			conditionBuilder.add_condition(CreateProtocolCondition(endpoint.value().protocol));
		}

		if (!objectInstaller.addFilter(filterBuilder, conditionBuilder))
		{
			return false;
		}
	}
	return true;
}

bool PermitVpnTunnelService::apply(IObjectInstaller &objectInstaller)
{
	if (!m_potentialEndpoints.has_value())
	{
		return AddEndpointFilter(
			std::nullopt,
			MullvadGuids::Filter_Baseline_PermitVpnTunnelService_Ipv4_1(),
			MullvadGuids::Filter_Baseline_PermitVpnTunnelService_Ipv6_1(),
			objectInstaller
		);
	}
	AddEndpointFilter(
			std::make_optional<Endpoint>(m_potentialEndpoints.value().entryEndpoint),
			MullvadGuids::Filter_Baseline_PermitVpnTunnelService_Ipv4_1(),
			MullvadGuids::Filter_Baseline_PermitVpnTunnelService_Ipv6_1(),
			objectInstaller
		);
	if (m_potentialEndpoints.value().exitEndpoint.has_value())
	{
		AddEndpointFilter(
				m_potentialEndpoints.value().exitEndpoint.value(),
				MullvadGuids::Filter_Baseline_PermitVpnTunnelService_Ipv4_2(),
				MullvadGuids::Filter_Baseline_PermitVpnTunnelService_Ipv6_2(),
				objectInstaller
			);
	}
	return true;
}

}
