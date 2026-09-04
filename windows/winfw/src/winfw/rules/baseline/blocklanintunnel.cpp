#include "stdafx.h"
#include "blocklanintunnel.h"
#include <winfw/mullvadguids.h>
#include <winfw/lannetworks.h>
#include <libwfp/filterbuilder.h>
#include <libwfp/conditionbuilder.h>
#include <libwfp/ipnetwork.h>
#include <libwfp/conditions/conditioninterface.h>
#include <libwfp/conditions/conditionip.h>

using namespace wfp::conditions;

namespace rules::baseline
{

namespace
{

const auto WEIGHT = wfp::FilterBuilder::WeightClass::Max;

} // anonymous namespace

BlockLanInTunnel::BlockLanInTunnel(const std::wstring &tunnelInterfaceAlias)
	: m_tunnelInterfaceAlias(tunnelInterfaceAlias)
{
}

bool BlockLanInTunnel::apply(IObjectInstaller &objectInstaller)
{
	return applyIpv4(objectInstaller) && applyIpv6(objectInstaller);
}

bool BlockLanInTunnel::applyIpv4(IObjectInstaller &objectInstaller) const
{
	wfp::FilterBuilder filterBuilder;

	//
	// #1 Permit outbound connections to Mullvad's in-tunnel networks.
	//

	filterBuilder
		.key(MullvadGuids::Filter_Baseline_BlockLanInTunnel_PermitMullvad_Outbound_Ipv4())
		.name(L"Permit outbound connections to Mullvad in-tunnel networks (IPv4)")
		.description(L"This filter is part of a rule that blocks LAN traffic on the tunnel interface")
		.provider(MullvadGuids::Provider())
		.layer(FWPM_LAYER_ALE_AUTH_CONNECT_V4)
		.sublayer(MullvadGuids::SublayerBaseline())
		.weight(WEIGHT)
		.permit();

	wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_CONNECT_V4);

	conditionBuilder.add_condition(ConditionInterface::Alias(m_tunnelInterfaceAlias));

	for (const auto &network : g_ipv4InTunnelLanNets) {
		conditionBuilder.add_condition(ConditionIp::Remote(network));
	}

	if (!objectInstaller.addFilter(filterBuilder, conditionBuilder))
	{
		return false;
	}

	//
	// #2 Block outbound connections to the LAN.
	//

	filterBuilder
		.key(MullvadGuids::Filter_Baseline_BlockLanInTunnel_Outbound_Ipv4())
		.name(L"Block outbound connections to the LAN on tunnel interface (IPv4)")
		.weight(WEIGHT)
		.block();

	conditionBuilder.reset();

	conditionBuilder.add_condition(ConditionInterface::Alias(m_tunnelInterfaceAlias));

	for (const auto &network : g_ipv4LanNets) {
		conditionBuilder.add_condition(ConditionIp::Remote(network));
	}
	for (const auto &network : g_ipv4MulticastNets) {
		conditionBuilder.add_condition(ConditionIp::Remote(network));
	}

	return objectInstaller.addFilter(filterBuilder, conditionBuilder);
}

bool BlockLanInTunnel::applyIpv6(IObjectInstaller &objectInstaller) const
{
	wfp::FilterBuilder filterBuilder;

	//
	// #1 Permit outbound connections to Mullvad's in-tunnel networks.
	//

	filterBuilder
		.key(MullvadGuids::Filter_Baseline_BlockLanInTunnel_PermitMullvad_Outbound_Ipv6())
		.name(L"Permit outbound connections to Mullvad in-tunnel networks (IPv6)")
		.description(L"This filter is part of a rule that blocks LAN traffic on the tunnel interface")
		.provider(MullvadGuids::Provider())
		.layer(FWPM_LAYER_ALE_AUTH_CONNECT_V6)
		.sublayer(MullvadGuids::SublayerBaseline())
		.weight(WEIGHT)
		.permit();

	wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_CONNECT_V6);

	conditionBuilder.add_condition(ConditionInterface::Alias(m_tunnelInterfaceAlias));

	for (const auto &network : g_ipv6InTunnelLanNets) {
		conditionBuilder.add_condition(ConditionIp::Remote(network));
	}

	if (!objectInstaller.addFilter(filterBuilder, conditionBuilder))
	{
		return false;
	}

	//
	// #2 Block outbound connections to the LAN.
	//

	filterBuilder
		.key(MullvadGuids::Filter_Baseline_BlockLanInTunnel_Outbound_Ipv6())
		.name(L"Block outbound connections to the LAN on tunnel interface (IPv6)")
		.layer(FWPM_LAYER_ALE_AUTH_CONNECT_V6)
		.weight(WEIGHT)
		.block();

	conditionBuilder.reset();

	conditionBuilder.add_condition(ConditionInterface::Alias(m_tunnelInterfaceAlias));

	for (const auto &network : g_ipv6LanNets) {
		conditionBuilder.add_condition(ConditionIp::Remote(network));
	}
	for (const auto &network : g_ipv6MulticastNets) {
		conditionBuilder.add_condition(ConditionIp::Remote(network));
	}

	return objectInstaller.addFilter(filterBuilder, conditionBuilder);
}

}
