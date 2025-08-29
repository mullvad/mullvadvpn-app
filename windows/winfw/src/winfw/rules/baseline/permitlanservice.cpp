#include "stdafx.h"
#include "permitlanservice.h"
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

bool PermitLanService::apply(IObjectInstaller &objectInstaller)
{
	return applyIpv4(objectInstaller) && applyIpv6(objectInstaller);
}

bool PermitLanService::applyIpv4(IObjectInstaller &objectInstaller) const
{
	wfp::FilterBuilder filterBuilder;

	//
	// #1 Permit inbound connections on LAN.
	//

	filterBuilder
		.key(MullvadGuids::Filter_Baseline_PermitLanService_Inbound_Ipv4())
		.name(L"Permit inbound connections on LAN (IPv4)")
		.description(L"This filter is part of a rule that permits hosting services in a LAN environment")
		.provider(MullvadGuids::Provider())
		.layer(FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V4)
		.sublayer(MullvadGuids::SublayerBaseline())
		.weight(wfp::FilterBuilder::WeightClass::Medium)
		.permit();

	wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V4);

	for (const auto &network : g_ipv4LanNets) {
		conditionBuilder.add_condition(ConditionIp::Remote(network));
	}

	return objectInstaller.addFilter(filterBuilder, conditionBuilder);
}

bool PermitLanService::applyIpv6(IObjectInstaller &objectInstaller) const
{
	wfp::FilterBuilder filterBuilder;

	//
	// #1 Permit inbound connections on LAN.
	//

	filterBuilder
		.key(MullvadGuids::Filter_Baseline_PermitLanService_Inbound_Ipv6())
		.name(L"Permit inbound connections on LAN (IPv6)")
		.description(L"This filter is part of a rule that permits hosting services in a LAN environment")
		.provider(MullvadGuids::Provider())
		.layer(FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V6)
		.sublayer(MullvadGuids::SublayerBaseline())
		.weight(wfp::FilterBuilder::WeightClass::Medium)
		.permit();

	wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V6);

	for (const auto &network : g_ipv6LanNets) {
		conditionBuilder.add_condition(ConditionIp::Remote(network));
	}

	return objectInstaller.addFilter(filterBuilder, conditionBuilder);
}

}
