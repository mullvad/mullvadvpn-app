#include "stdafx.h"
#include "permitlan.h"
#include <winfw/mullvadguids.h>
#include <libwfp/filterbuilder.h>
#include <libwfp/conditionbuilder.h>
#include <libwfp/ipaddress.h>
#include <libwfp/ipnetwork.h>
#include <libwfp/conditions/conditionip.h>
#include <winfw/lannetworks.h>

using namespace wfp::conditions;

namespace rules::baseline
{

bool PermitLan::apply(IObjectInstaller &objectInstaller)
{
	return applyIpv4(objectInstaller) && applyIpv6(objectInstaller);
}

bool PermitLan::applyIpv4(IObjectInstaller &objectInstaller) const
{
	wfp::FilterBuilder filterBuilder;

	//
	// #1 Permit outbound connections on LAN.
	//

	filterBuilder
		.key(MullvadGuids::Filter_Baseline_PermitLan_Outbound_Ipv4())
		.name(L"Permit outbound connections on LAN (IPv4)")
		.description(L"This filter is part of a rule that permits LAN traffic")
		.provider(MullvadGuids::Provider())
		.layer(FWPM_LAYER_ALE_AUTH_CONNECT_V4)
		.sublayer(MullvadGuids::SublayerBaseline())
		.weight(wfp::FilterBuilder::WeightClass::Medium)
		.permit();

	wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_CONNECT_V4);

	for (const auto &network : g_ipv4LanNets) {
		conditionBuilder.add_condition(ConditionIp::Remote(network));
	}

	if (!objectInstaller.addFilter(filterBuilder, conditionBuilder))
	{
		return false;
	}

	//
	// #2 Permit outbound multicast on LAN.
	//

	filterBuilder
		.key(MullvadGuids::Filter_Baseline_PermitLan_Outbound_Multicast_Ipv4())
		.name(L"Permit outbound multicast on LAN (IPv4)");

	conditionBuilder.reset();

	for (const auto &network : g_ipv4MulticastNets) {
		conditionBuilder.add_condition(ConditionIp::Remote(network));
	}

	return objectInstaller.addFilter(filterBuilder, conditionBuilder);
}

bool PermitLan::applyIpv6(IObjectInstaller &objectInstaller) const
{
	wfp::FilterBuilder filterBuilder;

	//
	// #1 Permit outbound connections on LAN.
	//

	filterBuilder
		.key(MullvadGuids::Filter_Baseline_PermitLan_Outbound_Ipv6())
		.name(L"Permit outbound connections on LAN (IPv6)")
		.description(L"This filter is part of a rule that permits LAN traffic")
		.provider(MullvadGuids::Provider())
		.layer(FWPM_LAYER_ALE_AUTH_CONNECT_V6)
		.sublayer(MullvadGuids::SublayerBaseline())
		.weight(wfp::FilterBuilder::WeightClass::Medium)
		.permit();

	wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_CONNECT_V6);

	for (const auto &network : g_ipv6LanNets) {
		conditionBuilder.add_condition(ConditionIp::Remote(network));
	}

	if (!objectInstaller.addFilter(filterBuilder, conditionBuilder))
	{
		return false;
	}

	//
	// #2 Permit outbound multicast on LAN.
	//

	filterBuilder
		.key(MullvadGuids::Filter_Baseline_PermitLan_Outbound_Multicast_Ipv6())
		.name(L"Permit outbound multicast on LAN (IPv6)");

	conditionBuilder.reset();

	for (const auto &network : g_ipv6MulticastNets) {
		conditionBuilder.add_condition(ConditionIp::Remote(network));
	}

	return objectInstaller.addFilter(filterBuilder, conditionBuilder);
}

}
