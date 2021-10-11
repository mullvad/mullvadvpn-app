#include "stdafx.h"
#include "permitndp.h"
#include <winfw/mullvadguids.h>
#include <libwfp/filterbuilder.h>
#include <libwfp/conditionbuilder.h>
#include <libwfp/ipaddress.h>
#include <libwfp/ipnetwork.h>
#include <libwfp/conditions/conditionprotocol.h>
#include <libwfp/conditions/conditionicmp.h>
#include <libwfp/conditions/conditionip.h>

using namespace wfp::conditions;

namespace rules::baseline
{

bool PermitNdp::apply(IObjectInstaller &objectInstaller)
{
	const wfp::IpNetwork linkLocal(wfp::IpAddress::Literal6({ 0xFE80, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0 }), 10);
	const wfp::IpAddress::Literal6 linkLocalRouterMulticast{ 0xFF02, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x2 };
	const wfp::IpNetwork solicitedNodeMulticast(wfp::IpAddress::Literal6({ 0xFF02, 0, 0, 0, 0, 1, 0xFF00, 0 }), 104);

	wfp::FilterBuilder filterBuilder;

	//
	// #1 Permit outbound router solicitation.
	//

	filterBuilder
		.key(MullvadGuids::Filter_Baseline_PermitNdp_Outbound_Router_Solicitation())
		.name(L"Permit outbound NDP router solicitation")
		.description(L"This filter is part of a rule that permits the most central parts of NDP")
		.provider(MullvadGuids::Provider())
		.layer(FWPM_LAYER_ALE_AUTH_CONNECT_V6)
		.sublayer(MullvadGuids::SublayerBaseline())
		.weight(wfp::FilterBuilder::WeightClass::Medium)
		.permit();

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
	// #2 Permit inbound router advertisement.
	//

	filterBuilder
		.key(MullvadGuids::Filter_Baseline_PermitNdp_Inbound_Router_Advertisement())
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
	// #3 Permit inbound redirect message.
	//

	filterBuilder
		.key(MullvadGuids::Filter_Baseline_PermitNdp_Inbound_Redirect())
		.name(L"Permit inbound NDP redirect")
		.layer(FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V6);

	{
		wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V6);

		conditionBuilder.add_condition(ConditionProtocol::IcmpV6());
		conditionBuilder.add_condition(ConditionIcmp::Type(137));
		conditionBuilder.add_condition(ConditionIcmp::Code(0));
		conditionBuilder.add_condition(ConditionIp::Remote(linkLocal));

		if (!objectInstaller.addFilter(filterBuilder, conditionBuilder))
		{
			return false;
		}
	}

	//
	// #4 Permit outbound neighbor solicitation.
	//

	filterBuilder
		.key(MullvadGuids::Filter_Baseline_PermitNdp_Outbound_Neighbor_Solicitation())
		.name(L"Permit outbound NDP neighbor solicitation")
		.layer(FWPM_LAYER_ALE_AUTH_CONNECT_V6);

	{
		wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_CONNECT_V6);

		conditionBuilder.add_condition(ConditionProtocol::IcmpV6());
		conditionBuilder.add_condition(ConditionIcmp::Type(135));
		conditionBuilder.add_condition(ConditionIcmp::Code(0));
		conditionBuilder.add_condition(ConditionIp::Remote(solicitedNodeMulticast));
		conditionBuilder.add_condition(ConditionIp::Remote(linkLocal));

		if (!objectInstaller.addFilter(filterBuilder, conditionBuilder))
		{
			return false;
		}
	}

	//
	// #5 Permit inbound neighbor solicitation.
	//

	filterBuilder
		.key(MullvadGuids::Filter_Baseline_PermitNdp_Inbound_Neighbor_Solicitation())
		.name(L"Permit inbound NDP neighbor solicitation")
		.layer(FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V6);

	{
		wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V6);

		conditionBuilder.add_condition(ConditionProtocol::IcmpV6());
		conditionBuilder.add_condition(ConditionIcmp::Type(135));
		conditionBuilder.add_condition(ConditionIcmp::Code(0));
		conditionBuilder.add_condition(ConditionIp::Remote(linkLocal));

		if (!objectInstaller.addFilter(filterBuilder, conditionBuilder))
		{
			return false;
		}
	}

	//
	// #6 Permit outbound neighbor advertisement.
	//

	filterBuilder
		.key(MullvadGuids::Filter_Baseline_PermitNdp_Outbound_Neighbor_Advertisement())
		.name(L"Permit outbound NDP neighbor advertisement")
		.layer(FWPM_LAYER_ALE_AUTH_CONNECT_V6);

	{
		wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_CONNECT_V6);

		conditionBuilder.add_condition(ConditionProtocol::IcmpV6());
		conditionBuilder.add_condition(ConditionIcmp::Type(136));
		conditionBuilder.add_condition(ConditionIcmp::Code(0));
		conditionBuilder.add_condition(ConditionIp::Remote(linkLocal));

		if (!objectInstaller.addFilter(filterBuilder, conditionBuilder))
		{
			return false;
		}
	}

	//
	// #7 Permit inbound neighbor advertisement.
	//

	filterBuilder
		.key(MullvadGuids::Filter_Baseline_PermitNdp_Inbound_Neighbor_Advertisement())
		.name(L"Permit inbound NDP neighbor advertisement")
		.layer(FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V6);

	wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V6);

	conditionBuilder.add_condition(ConditionProtocol::IcmpV6());
	conditionBuilder.add_condition(ConditionIcmp::Type(136));
	conditionBuilder.add_condition(ConditionIcmp::Code(0));

	return objectInstaller.addFilter(filterBuilder, conditionBuilder);
}

}
