#include "stdafx.h"
#include "permitloopback.h"
#include <winfw/mullvadguids.h>
#include <libwfp/filterbuilder.h>
#include <libwfp/conditionbuilder.h>
#include <libwfp/conditions/conditionloopback.h>

using namespace wfp::conditions;

namespace rules::baseline
{

bool PermitLoopback::apply(IObjectInstaller &objectInstaller)
{
	wfp::FilterBuilder filterBuilder;

	//
	// #1 Permit outbound connections, IPv4.
	//

	filterBuilder
		.key(MullvadGuids::Filter_Baseline_PermitLoopback_Outbound_Ipv4())
		.name(L"Permit outbound connections on loopback (IPv4)")
		.description(L"This filter is part of a rule that permits all loopback traffic")
		.provider(MullvadGuids::Provider())
		.layer(FWPM_LAYER_ALE_AUTH_CONNECT_V4)
		.sublayer(MullvadGuids::SublayerBaseline())
		.weight(wfp::FilterBuilder::WeightClass::Medium)
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
	// #2 Permit inbound connections, IPv4.
	//

	filterBuilder
		.key(MullvadGuids::Filter_Baseline_PermitLoopback_Inbound_Ipv4())
		.name(L"Permit inbound connections on loopback (IPv4)")
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
	// #3 Permit outbound connections, IPv6.
	//

	filterBuilder
		.key(MullvadGuids::Filter_Baseline_PermitLoopback_Outbound_Ipv6())
		.name(L"Permit outbound connections on loopback (IPv6)")
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
	// #4 Permit inbound connections, IPv6.
	//

	filterBuilder
		.key(MullvadGuids::Filter_Baseline_PermitLoopback_Inbound_Ipv6())
		.name(L"Permit inbound connections on loopback (IPv6)")
		.layer(FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V6);

	wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V6);

	conditionBuilder.add_condition(std::make_unique<ConditionLoopback>());

	return objectInstaller.addFilter(filterBuilder, conditionBuilder);
}

}
