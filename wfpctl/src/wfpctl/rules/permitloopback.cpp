#include "stdafx.h"
#include "permitloopback.h"
#include "wfpctl/mullvadguids.h"
#include "libwfp/filterbuilder.h"
#include "libwfp/conditionbuilder.h"
#include "libwfp/conditions/conditionloopback.h"

using namespace wfp::conditions;

namespace rules
{

bool PermitLoopback::apply(IObjectInstaller &objectInstaller)
{
	wfp::FilterBuilder filterBuilder;

	//
	// #1 permit outbound connections, ipv4
	//

	filterBuilder
		.key(MullvadGuids::FilterPermitLoopback_Outbound_Ipv4())
		.name(L"Permit outbound connections on loopback")
		.description(L"This filter is part of a rule that permits all loopback traffic")
		.provider(MullvadGuids::Provider())
		.layer(FWPM_LAYER_ALE_AUTH_CONNECT_V4)
		.sublayer(MullvadGuids::SublayerWhitelist())
		.weight(wfp::FilterBuilder::WeightClass::Max)
		.permit();

	{
		wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_CONNECT_V4);

		conditionBuilder.add_condition(std::make_unique<ConditionLoopback>());

		if (false == objectInstaller.addFilter(filterBuilder, conditionBuilder))
		{
			return false;
		}
	}

	//
	// #2 permit outbound connections, ipv6
	//

	filterBuilder
		.key(MullvadGuids::FilterPermitLoopback_Outbound_Ipv6())
		.layer(FWPM_LAYER_ALE_AUTH_CONNECT_V6);

	{
		wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_CONNECT_V6);

		conditionBuilder.add_condition(std::make_unique<ConditionLoopback>());

		if (false == objectInstaller.addFilter(filterBuilder, conditionBuilder))
		{
			return false;
		}
	}

	//
	// #3 permit inbound connections, ipv4
	//

	filterBuilder
		.key(MullvadGuids::FilterPermitLoopback_Inbound_Ipv4())
		.name(L"Permit inbound connections on loopback")
		.layer(FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V4);

	{
		wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V4);

		conditionBuilder.add_condition(std::make_unique<ConditionLoopback>());

		if (false == objectInstaller.addFilter(filterBuilder, conditionBuilder))
		{
			return false;
		}
	}

	//
	// #4 permit inbound connections, ipv6
	//

	filterBuilder
		.key(MullvadGuids::FilterPermitLoopback_Inbound_Ipv6())
		.layer(FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V6);

	wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V6);

	conditionBuilder.add_condition(std::make_unique<ConditionLoopback>());

	return objectInstaller.addFilter(filterBuilder, conditionBuilder);
}

}
