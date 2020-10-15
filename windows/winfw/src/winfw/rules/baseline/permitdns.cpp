#include "stdafx.h"
#include "permitdns.h"
#include <winfw/mullvadguids.h>
#include <winfw/rules/ports.h>
#include <libwfp/filterbuilder.h>
#include <libwfp/conditionbuilder.h>
#include <libwfp/conditions/conditionport.h>

using namespace wfp::conditions;

namespace rules::baseline
{

bool PermitDns::apply(IObjectInstaller &objectInstaller)
{
	wfp::FilterBuilder filterBuilder;

	//
	// #1 Permit outbound DNS, IPv4.
	//

	filterBuilder
		.key(MullvadGuids::Filter_Baseline_PermitDns_Outbound_Ipv4())
		.name(L"Permit outbound connections to DNS server (IPv4)")
		.description(L"This filter is part of a rule that permits outbound DNS")
		.provider(MullvadGuids::Provider())
		.layer(FWPM_LAYER_ALE_AUTH_CONNECT_V4)
		.sublayer(MullvadGuids::SublayerBaseline())
		.weight(wfp::FilterBuilder::WeightClass::Medium)
		.permit();

	wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_CONNECT_V4);

	conditionBuilder.add_condition(ConditionPort::Remote(DNS_SERVER_PORT));

	if (false == objectInstaller.addFilter(filterBuilder, conditionBuilder))
	{
		return false;
	}

	//
	// #2 Permit outbound DNS, IPv6.
	//

	filterBuilder
		.key(MullvadGuids::Filter_Baseline_PermitDns_Outbound_Ipv6())
		.name(L"Permit outbound connections to DNS server (IPv6)")
		.layer(FWPM_LAYER_ALE_AUTH_CONNECT_V6);

	conditionBuilder.reset(FWPM_LAYER_ALE_AUTH_CONNECT_V6);
	conditionBuilder.add_condition(ConditionPort::Remote(DNS_SERVER_PORT));

	return objectInstaller.addFilter(filterBuilder, conditionBuilder);
}

}
