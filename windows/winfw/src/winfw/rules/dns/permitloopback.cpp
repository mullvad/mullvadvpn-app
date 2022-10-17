#include "stdafx.h"
#include "permitloopback.h"
#include <winfw/mullvadguids.h>
#include <winfw/rules/ports.h>
#include <libwfp/filterbuilder.h>
#include <libwfp/conditionbuilder.h>
#include <libwfp/conditions/conditionloopback.h>
#include <libwfp/conditions/conditionport.h>

using namespace wfp::conditions;

namespace rules::dns
{

bool PermitLoopback::apply(IObjectInstaller &objectInstaller)
{
	wfp::FilterBuilder filterBuilder;

	//
	// #1 Permit outbound connections, IPv4.
	//

	filterBuilder
		.key(MullvadGuids::Filter_Dns_PermitLoopback_Outbound_Ipv4())
		.name(L"Permit loopback DNS traffic (IPv4)")
		.description(L"This filter is part of a rule that permits loopback DNS traffic")
		.provider(MullvadGuids::Provider())
		.layer(FWPM_LAYER_ALE_AUTH_CONNECT_V4)
		.sublayer(MullvadGuids::SublayerDns())
		.weight(wfp::FilterBuilder::WeightClass::Medium)
		.permit();

	{
		wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_CONNECT_V4);

		conditionBuilder.add_condition(std::make_unique<ConditionLoopback>());
		conditionBuilder.add_condition(ConditionPort::Remote(DNS_SERVER_PORT));

		if (false == objectInstaller.addFilter(filterBuilder, conditionBuilder))
		{
			return false;
		}
	}

	//
	// #2 Permit outbound connections, IPv6.
	//

	filterBuilder
		.key(MullvadGuids::Filter_Dns_PermitLoopback_Outbound_Ipv6())
		.name(L"Permit loopback DNS traffic (IPv6)")
		.layer(FWPM_LAYER_ALE_AUTH_CONNECT_V6);

	wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_CONNECT_V6);

	conditionBuilder.add_condition(std::make_unique<ConditionLoopback>());
	conditionBuilder.add_condition(ConditionPort::Remote(DNS_SERVER_PORT));

	return objectInstaller.addFilter(filterBuilder, conditionBuilder);
}

}
