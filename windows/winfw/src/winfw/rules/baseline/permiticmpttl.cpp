#include "stdafx.h"
#include "permiticmpttl.h"
#include <winfw/mullvadguids.h>
#include <winfw/rules/shared.h>
#include <libwfp/filterbuilder.h>
#include <libwfp/conditionbuilder.h>
#include <libwfp/conditions/conditionprotocol.h>
#include <libwfp/conditions/conditionip.h>
#include <libwfp/conditions/conditionicmp.h>
#include <libwfp/conditions/conditionport.h>
#include <libwfp/conditions/conditionapplication.h>
#include <libcommon/error.h>

using namespace wfp::conditions;

namespace rules::baseline
{

PermitIcmpTtl::PermitIcmpTtl(const std::vector<std::wstring> &relayClients)
	: m_relayClients(relayClients)
{
}

bool PermitIcmpTtl::apply(IObjectInstaller &objectInstaller)
{
	wfp::FilterBuilder filterBuilder;

	//
	// Permit incoming ICMP TimeExceeded packets to the daemon.
	//

	filterBuilder
		.key(MullvadGuids::Filter_Baseline_PermitIcmpTtl())
		.name(L"Permit inbound ICMP Time Exceeded packets from a given endpoint")
		.description(L"This filter is part of a rule that permits traffic from a specific endpoint")
		.provider(MullvadGuids::Provider())
		.layer(FWPM_LAYER_INBOUND_ICMP_ERROR_V4)
		.sublayer(MullvadGuids::SublayerBaseline())
		.weight(wfp::FilterBuilder::WeightClass::Max)
		.permit();

	{
		wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_INBOUND_ICMP_ERROR_V4);
		for(auto relayClient : m_relayClients) {
			conditionBuilder.add_condition(std::make_unique<ConditionApplication>(relayClient));
		}

		if (!objectInstaller.addFilter(filterBuilder, conditionBuilder)) {
			return false;
		}
	}

	//
	// Permit incoming ICMPv6 TimeExceeded packets to the daemon.
	//

	filterBuilder
		.key(MullvadGuids::Filter_Baseline_PermitIcmpTtlV6())
		.layer(FWPM_LAYER_INBOUND_ICMP_ERROR_V6);

	wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_INBOUND_ICMP_ERROR_V6);
	for(auto relayClient : m_relayClients) {
		conditionBuilder.add_condition(std::make_unique<ConditionApplication>(relayClient));
	}

	return objectInstaller.addFilter(filterBuilder, conditionBuilder);
}

}
