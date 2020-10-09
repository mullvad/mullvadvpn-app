#include "stdafx.h"
#include "blockall.h"
#include <winfw/mullvadguids.h>
#include <libwfp/filterbuilder.h>
#include <libwfp/nullconditionbuilder.h>

namespace rules::persistent
{

bool BlockAll::apply(IObjectInstaller &objectInstaller)
{
	wfp::FilterBuilder filterBuilder;

	//
	// Add boot-time filters (i.e., filters applied before BFE starts)
	//

	filterBuilder
		.key(MullvadGuids::Filter_Boottime_BlockAll_Outbound_Ipv4())
		.name(L"Block all outbound connections (IPv4)")
		.description(L"This filter is part of a rule that restricts inbound and outbound traffic")
		.provider(MullvadGuids::ProviderPersistent())
		.layer(FWPM_LAYER_ALE_AUTH_CONNECT_V4)
		.sublayer(MullvadGuids::SublayerPersistent())
		.weight(wfp::FilterBuilder::WeightClass::Max)
		.boottime()
		.block();

	wfp::NullConditionBuilder nullConditionBuilder;

	if (false == objectInstaller.addFilter(filterBuilder, nullConditionBuilder))
	{
		return false;
	}

	filterBuilder
		.key(MullvadGuids::Filter_Boottime_BlockAll_Inbound_Ipv4())
		.name(L"Block all inbound connections (IPv4)")
		.layer(FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V4);

	if (false == objectInstaller.addFilter(filterBuilder, nullConditionBuilder))
	{
		return false;
	}

	filterBuilder
		.key(MullvadGuids::Filter_Boottime_BlockAll_Outbound_Ipv6())
		.name(L"Block all outbound connections (IPv6)")
		.layer(FWPM_LAYER_ALE_AUTH_CONNECT_V6);

	if (false == objectInstaller.addFilter(filterBuilder, nullConditionBuilder))
	{
		return false;
	}

	filterBuilder
		.key(MullvadGuids::Filter_Boottime_BlockAll_Inbound_Ipv6())
		.name(L"Block all inbound connections (IPv6)")
		.layer(FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V6);

	if (false == objectInstaller.addFilter(filterBuilder, nullConditionBuilder))
	{
		return false;
	}

	//
	// Add persistent filters (i.e., filters applied when BFE has started)
	//

	wfp::FilterBuilder persistentFilterBuilder;

	persistentFilterBuilder
		.key(MullvadGuids::Filter_Persistent_BlockAll_Outbound_Ipv4())
		.name(L"Block all outbound connections (IPv4)")
		.description(L"This filter is part of a rule that restricts inbound and outbound traffic")
		.provider(MullvadGuids::ProviderPersistent())
		.layer(FWPM_LAYER_ALE_AUTH_CONNECT_V4)
		.sublayer(MullvadGuids::SublayerPersistent())
		.weight(wfp::FilterBuilder::WeightClass::Max)
		.persistent()
		.block();

	if (false == objectInstaller.addFilter(persistentFilterBuilder, nullConditionBuilder))
	{
		return false;
	}

	persistentFilterBuilder
		.key(MullvadGuids::Filter_Persistent_BlockAll_Inbound_Ipv4())
		.name(L"Block all inbound connections (IPv4)")
		.layer(FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V4);

	if (false == objectInstaller.addFilter(persistentFilterBuilder, nullConditionBuilder))
	{
		return false;
	}

	persistentFilterBuilder
		.key(MullvadGuids::Filter_Persistent_BlockAll_Outbound_Ipv6())
		.name(L"Block all outbound connections (IPv6)")
		.layer(FWPM_LAYER_ALE_AUTH_CONNECT_V6);

	if (false == objectInstaller.addFilter(persistentFilterBuilder, nullConditionBuilder))
	{
		return false;
	}

	persistentFilterBuilder
		.key(MullvadGuids::Filter_Persistent_BlockAll_Inbound_Ipv6())
		.name(L"Block all inbound connections (IPv6)")
		.layer(FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V6);

	return objectInstaller.addFilter(persistentFilterBuilder, nullConditionBuilder);
}

}
