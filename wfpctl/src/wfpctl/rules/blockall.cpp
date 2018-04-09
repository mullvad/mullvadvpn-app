#include "stdafx.h"
#include "blockall.h"
#include "wfpctl/mullvadguids.h"
#include "libwfp/filterbuilder.h"
#include "libwfp/nullconditionbuilder.h"

namespace rules
{

bool BlockAll::apply(IObjectInstaller &objectInstaller)
{
	wfp::FilterBuilder filterBuilder;

	//
	// #1 block outbound connections, ipv4
	//

	filterBuilder
		.key(MullvadGuids::FilterBlockAll_Outbound_Ipv4())
		.name(L"Block all outbound connections")
		.description(L"This filter is part of a rule that restricts inbound and outbound traffic")
		.provider(MullvadGuids::Provider())
		.layer(FWPM_LAYER_ALE_AUTH_CONNECT_V4)
		.sublayer(MullvadGuids::SublayerWhitelist())
		.weight(wfp::FilterBuilder::WeightClass::Min)
		.block();

	wfp::NullConditionBuilder nullConditionBuilder;

	if (false == objectInstaller.addFilter(filterBuilder, nullConditionBuilder))
	{
		return false;
	}

	//
	// #2 block outbound connections, ipv6
	//

	filterBuilder
		.key(MullvadGuids::FilterBlockAll_Outbound_Ipv6())
		.layer(FWPM_LAYER_ALE_AUTH_CONNECT_V6);

	if (false == objectInstaller.addFilter(filterBuilder, nullConditionBuilder))
	{
		return false;
	}

	//
	// #3 block inbound connections, ipv4
	//

	filterBuilder
		.key(MullvadGuids::FilterBlockAll_Inbound_Ipv4())
		.name(L"Block all inbound connections")
		.layer(FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V4);

	if (false == objectInstaller.addFilter(filterBuilder, nullConditionBuilder))
	{
		return false;
	}

	//
	// #4 block inbound connections, ipv6
	//

	filterBuilder
		.key(MullvadGuids::FilterBlockAll_Inbound_Ipv6())
		.layer(FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V6);

	return objectInstaller.addFilter(filterBuilder, nullConditionBuilder);
}

}
