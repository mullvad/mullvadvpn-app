#include "stdafx.h"
#include "permitdhcp.h"
#include "wfpctl/mullvadguids.h"
#include "libwfp/filterbuilder.h"
#include "libwfp/conditionbuilder.h"
#include "libwfp/ipaddress.h"
#include "libwfp/conditions/conditionprotocol.h"
#include "libwfp/conditions/conditionport.h"
#include "libwfp/conditions/conditionip.h"
#include "libwfp/conditions/conditionport.h"

using namespace wfp::conditions;

namespace rules
{

bool PermitDhcp::apply(IObjectInstaller &objectInstaller)
{
	//
	// First UDP packet for a unique [remote address, port] tuple is mapped into:
	//
	// outbound: FWPM_LAYER_ALE_AUTH_CONNECT_V{4|6}
	// inbound: FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V{4|6}
	//

	wfp::FilterBuilder filterBuilder;

	//
	// #1 permit outbound DHCP request
	//

	filterBuilder
		.key(MullvadGuids::FilterPermitDhcp_Outbound_Request())
		.name(L"Permit outbound DHCP request")
		.description(L"This filter is part of a rule that permits DHCP client traffic")
		.provider(MullvadGuids::Provider())
		.layer(FWPM_LAYER_ALE_AUTH_CONNECT_V4)
		.sublayer(MullvadGuids::SublayerWhitelist())
		.weight(wfp::FilterBuilder::WeightClass::Max)
		.permit();

	{
		wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_CONNECT_V4);

		conditionBuilder.add_condition(ConditionProtocol::Udp());
		conditionBuilder.add_condition(ConditionPort::Local(68));
		conditionBuilder.add_condition(ConditionIp::Remote(wfp::IpAddress::Literal({ 255, 255, 255, 255 })));
		conditionBuilder.add_condition(ConditionPort::Remote(67));

		if (!objectInstaller.addFilter(filterBuilder, conditionBuilder))
		{
			return false;
		}
	}

	//
	// #2 permit inbound DHCP response
	//

	filterBuilder
		.key(MullvadGuids::FilterPermitDhcp_Inbound_Response())
		.name(L"Permit inbound DHCP response")
		.layer(FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V4);

	wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V4);

	conditionBuilder.add_condition(ConditionProtocol::Udp());
	conditionBuilder.add_condition(ConditionPort::Remote(67));
	conditionBuilder.add_condition(ConditionPort::Local(68));

	return objectInstaller.addFilter(filterBuilder, conditionBuilder);
}

}
