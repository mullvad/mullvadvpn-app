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
	const std::optional<Endpoints> &potentialEndpoints,
)
	: m_tunnelInterfaceAlias(tunnelInterfaceAlias)
	, m_potentialEndpoints(potentialEndpoints)
{
}

bool PermitVpnTunnel::add_endpoint_filter(std::optional<Endpoint> &endpoint, GUID ipv4Guid, GUID ipv6Guid, wfp::FilterBuilder &filterBuilder)
{
    if (!endpoint.has_value() || endpoint.ip.type() == wfp::IpAddress::Ipv4)
    {
        filterBuilder
            .key(guid)
            .name(L"Permit outbound connections on tunnel interface (IPv4)")
            .layer(FWPM_LAYER_ALE_AUTH_CONNECT_V4);
    
        wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_CONNECT_V4);
    
        conditionBuilder.add_condition(ConditionInterface::Alias(m_tunnelInterfaceAlias));
        if (endpoint.has_value())
        {
            conditionBuilder.add_condition(ConditionIp::Remote(endpoint.ip));
            conditionBuilder.add_condition(ConditionPort::Remote(endpoint.port));
            conditionBuilder.add_condition(CreateProtocolCondition(endpoint.protocol));
        }
    
        if (!objectInstaller.addFilter(filterBuilder, conditionBuilder))
        {
            return false;
        }
    }
    
    if (!endpoint.has_value() || endpoint.ip.type() == wfp::IpAddress::Ipv6)
    {
        filterBuilder
            .key(guid)
            .name(L"Permit outbound connections on tunnel interface (IPv6)")
            .layer(FWPM_LAYER_ALE_AUTH_CONNECT_V6);
    
        wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_CONNECT_V6);
    
        conditionBuilder.add_condition(ConditionInterface::Alias(m_tunnelInterfaceAlias));
        if (endpoint.has_value())
        {
            conditionBuilder.add_condition(ConditionIp::Remote(endpoint.ip));
            conditionBuilder.add_condition(ConditionPort::Remote(endpoint.port));
            conditionBuilder.add_condition(CreateProtocolCondition(endpoint.protocol));
        }
    
        if (!objectInstaller.addFilter(filterBuilder, conditionBuilder))
        {
            return false;
        }
    }
    return true;
}
}

bool PermitVpnTunnel::apply(IObjectInstaller &objectInstaller)
{
	wfp::FilterBuilder filterBuilder;

	filterBuilder
		.description(L"This filter is part of a rule that permits communications inside the VPN tunnel")
		.provider(MullvadGuids::Provider())
		.sublayer(MullvadGuids::SublayerBaseline())
		.weight(wfp::FilterBuilder::WeightClass::Medium)
		.permit();

    if (!m_potentialEndpoints.has_value()) {
        return add_endpoint_filter(
                    std::nullopt,
                    MullvadGuids::Filter_Baseline_PermitVpnTunnel_Outbound_Ipv4_Entry(),
                    MullvadGuids::Filter_Baseline_PermitVpnTunnel_Outbound_Ipv6_Entry(),
                    filterBuilder
                );
    } else {
        add_endpoint_filter(
                std::make_optional<Endpoint>(m_potentialEndpoints.entryEndpoint),
                MullvadGuids::Filter_Baseline_PermitVpnTunnel_Outbound_Ipv4_Entry(),
                MullvadGuids::Filter_Baseline_PermitVpnTunnel_Outbound_Ipv6_Entry(),
                filterBuilder
           );
        if (m_potentialEndpoints.exitEndpoint.has_value())
        {
            add_endpoint_filter(
                    m_potentialEndpoints.exitEndpoint,
                    MullvadGuids::Filter_Baseline_PermitVpnTunnel_Outbound_Ipv4_Exit(),
                    MullvadGuids::Filter_Baseline_PermitVpnTunnel_Outbound_Ipv6_Exit(),
                    filterBuilder
               );
        }
    }
	return true;
}

}
