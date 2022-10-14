#include "stdafx.h"
#include "blockall.h"
#include <winfw/mullvadguids.h>
#include <winfw/rules/ports.h>
#include <libwfp/filterbuilder.h>
#include <libwfp/conditionbuilder.h>
#include <libwfp/conditions/conditionport.h>

using namespace wfp::conditions;

namespace rules::dns
{

bool BlockAll::apply(IObjectInstaller &objectInstaller)
{
	wfp::FilterBuilder filterBuilder;

	//
	// #1 Block outbound DNS, IPv4.
	//

	filterBuilder
		.key(MullvadGuids::Filter_Dns_BlockAll_Outbound_Ipv4())
		.name(L"Block outbound DNS (IPv4)")
		.description(L"This filter is part of a rule that blocks DNS requests")
		.provider(MullvadGuids::Provider())
		.layer(FWPM_LAYER_ALE_AUTH_CONNECT_V4)
		.sublayer(MullvadGuids::SublayerDns())
		.weight(wfp::FilterBuilder::WeightClass::Min)
		.block();

	wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_CONNECT_V4);
	conditionBuilder.add_condition(ConditionPort::Remote(DNS_SERVER_PORT));

	if (false == objectInstaller.addFilter(filterBuilder, conditionBuilder))
	{
		return false;
	}

	//
	// #2 Block outbound DNS, IPv6.
	//

	filterBuilder
		.key(MullvadGuids::Filter_Dns_BlockAll_Outbound_Ipv6())
		.name(L"Block outbound DNS (IPv6)")
		.layer(FWPM_LAYER_ALE_AUTH_CONNECT_V6);

	conditionBuilder.reset(FWPM_LAYER_ALE_AUTH_CONNECT_V6);
	conditionBuilder.add_condition(ConditionPort::Remote(DNS_SERVER_PORT));

	return objectInstaller.addFilter(filterBuilder, conditionBuilder);
}

}
