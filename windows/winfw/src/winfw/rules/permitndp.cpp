#include "stdafx.h"
#include "permitndp.h"
#include "winfw/mullvadguids.h"
#include "libwfp/filterbuilder.h"
#include "libwfp/conditionbuilder.h"
#include "libwfp/ipaddress.h"
#include "libwfp/ipnetwork.h"
#include "libwfp/conditions/conditionprotocol.h"
#include "libwfp/conditions/conditionicmp.h"
#include "libwfp/conditions/conditionip.h"

using namespace wfp::conditions;

namespace rules
{

bool PermitNdp::apply(IObjectInstaller &objectInstaller)
{
	const wfp::IpNetwork linkLocal(wfp::IpAddress::Literal6({ 0xFE80, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0 }), 10);
	const wfp::IpAddress::Literal6 linkLocalRouterMulticast{ 0xFF02, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x2 };

	wfp::FilterBuilder filterBuilder;

	//
	// #1 permit outbound router solicitation
	//

	filterBuilder
		.key(MullvadGuids::FilterPermitNdp_Outbound_Router_Solicitation())
		.name(L"Permit outbound NDP router solicitation")
		.layer(FWPM_LAYER_ALE_AUTH_CONNECT_V6);

	{
		wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_CONNECT_V6);

		conditionBuilder.add_condition(ConditionProtocol::IcmpV6());
		conditionBuilder.add_condition(ConditionIcmp::Type(133));
		conditionBuilder.add_condition(ConditionIcmp::Code(0));
		conditionBuilder.add_condition(ConditionIp::Remote(linkLocalRouterMulticast));

		if (!objectInstaller.addFilter(filterBuilder, conditionBuilder))
		{
			return false;
		}
	}

	//
	// #2 permit inbound router advertisement
	//

	filterBuilder
		.key(MullvadGuids::FilterPermitNdp_Inbound_Router_Advertisement())
		.name(L"Permit inbound NDP router advertisement")
		.layer(FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V6);

	{
		wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V6);

		conditionBuilder.add_condition(ConditionProtocol::IcmpV6());
		conditionBuilder.add_condition(ConditionIcmp::Type(134));
		conditionBuilder.add_condition(ConditionIcmp::Code(0));
		conditionBuilder.add_condition(ConditionIp::Remote(linkLocal));

		if (!objectInstaller.addFilter(filterBuilder, conditionBuilder))
		{
			return false;
		}
	}

	//
	// #3 permit inbound redirect message
	//

	filterBuilder
		.key(MullvadGuids::FilterPermitNdp_Inbound_Redirect())
		.name(L"Permit inbound NDP redirect")
		.layer(FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V6);

	wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V6);

	conditionBuilder.add_condition(ConditionProtocol::IcmpV6());
	conditionBuilder.add_condition(ConditionIcmp::Type(137));
	conditionBuilder.add_condition(ConditionIcmp::Code(0));
	conditionBuilder.add_condition(ConditionIp::Remote(linkLocal));

	return objectInstaller.addFilter(filterBuilder, conditionBuilder);
}

}
