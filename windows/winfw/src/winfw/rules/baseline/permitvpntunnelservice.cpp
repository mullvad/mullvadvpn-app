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
	const std::vector<PermitVpnTunnel::Endpoint> &endpoints
)
	: m_tunnelInterfaceAlias(tunnelInterfaceAlias)
	, m_tunnelEndpoints(endpoints)
{
}

bool PermitVpnTunnelService::apply(IObjectInstaller &objectInstaller)
{
	wfp::FilterBuilder filterBuilder;

	filterBuilder
		.description(L"This filter is part of a rule that permits hosting services that listen on the tunnel interface")
		.provider(MullvadGuids::Provider())
		.layer(FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V4)
		.sublayer(MullvadGuids::SublayerBaseline())
		.weight(wfp::FilterBuilder::WeightClass::Medium)
		.permit();

    if (m_tunnelEndpoints.empty()) {
        filterBuilder
            .key(MullvadGuids::Filter_Baseline_PermitVpnTunnelService_Ipv4())
            .name(L"Permit inbound connections on tunnel interface (IPv4)")
            .layer(FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V4);

        wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_CONNECT_V4);

        conditionBuilder.add_condition(ConditionInterface::Alias(m_tunnelInterfaceAlias));

        if (!objectInstaller.addFilter(filterBuilder, conditionBuilder))
        {
            return false;
        }

		filterBuilder
			.key(MullvadGuids::Filter_Baseline_PermitVpnTunnelService_Ipv6())
			.name(L"Permit inbound connections on tunnel interface (IPv6)")
			.layer(FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V6);

		wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_CONNECT_V6);

		conditionBuilder.add_condition(ConditionInterface::Alias(m_tunnelInterfaceAlias));

		return objectInstaller.addFilter(filterBuilder, conditionBuilder);
    } else {
        for (int i = 0; i < std::min(m_tunnelEndpoints.size(), 2); i++) {
            if (m_tunnelEndpoints[i].ip.type() == wfp::IpAddress::Ipv4)
            {
                GUID guid;
                if (i == 0) {
                    guid = MullvadGuids::Filter_Baseline_PermitVpnTunnel_Outbound_Ipv4_Entry();
                } else if (i == 1) {
                    guid = MullvadGuids::Filter_Baseline_PermitVpnTunnel_Outbound_Ipv4_Exit();
                } else {
                    return false;
                }

                filterBuilder
                    .key(guid)
                    .name(L"Permit inbound connections on tunnel interface (IPv4)")
                    .layer(FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V4);

                wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_CONNECT_V4);

                conditionBuilder.add_condition(ConditionInterface::Alias(m_tunnelInterfaceAlias));
                conditionBuilder.add_condition(ConditionIp::Remote(m_tunnelEndpoints[i].ip));
                conditionBuilder.add_condition(ConditionPort::Remote(m_tunnelEndpoint[i].port));
                conditionBuilder.add_condition(CreateProtocolCondition(m_tunnelEndpoint[i].protocol));

                if (!objectInstaller.addFilter(filterBuilder, conditionBuilder))
                {
                    return false;
                }
            }

            if (m_tunnelEndpoints[i].ip.type() == wfp::IpAddress::Ipv6)
            {
                GUID guid;
                if (i == 0) {
                    guid = MullvadGuids::Filter_Baseline_PermitVpnTunnel_Outbound_Ipv6_Entry();
                } else if (i == 1) {
                    guid = MullvadGuids::Filter_Baseline_PermitVpnTunnel_Outbound_Ipv6_Exit();
                } else {
                    return false;
                }

                filterBuilder
                    .key(guid)
                    .name(L"Permit inbound connections on tunnel interface (IPv6)")
                    .layer(FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V6);

                wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_CONNECT_V4);

                conditionBuilder.add_condition(ConditionInterface::Alias(m_tunnelInterfaceAlias));
                conditionBuilder.add_condition(ConditionIp::Remote(m_tunnelEndpoints[i].ip));
                conditionBuilder.add_condition(ConditionPort::Remote(m_tunnelEndpoint[i].port));
                conditionBuilder.add_condition(CreateProtocolCondition(m_tunnelEndpoint[i].protocol));

                if (!objectInstaller.addFilter(filterBuilder, conditionBuilder))
                {
                    return false;
                }
            }

        }
    }
	return true;

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
			conditionBuilder.add_condition(ConditionPort::Remote(m_tunnelOnlyEndpoint->port));
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
			conditionBuilder.add_condition(ConditionPort::Remote(m_tunnelOnlyEndpoint->port));
			conditionBuilder.add_condition(CreateProtocolCondition(m_tunnelOnlyEndpoint->protocol));
		}

		return objectInstaller.addFilter(filterBuilder, conditionBuilder);
	}

	return true;
}

}
