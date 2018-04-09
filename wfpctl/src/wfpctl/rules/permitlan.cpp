#include "stdafx.h"
#include "permitlan.h"
#include "wfpctl/mullvadguids.h"
#include "libwfp/filterbuilder.h"
#include "libwfp/conditionbuilder.h"
#include "libwfp/ipaddress.h"
#include "libwfp/conditions/conditionip.h"

using namespace wfp::conditions;

namespace rules
{

bool PermitLan::apply(IObjectInstaller &objectInstaller)
{
	wfp::FilterBuilder filterBuilder;

	//
	// #1 locally-initiated on 10/8
	//

	filterBuilder
		.key(MullvadGuids::FilterPermitLan_10_8())
		.name(L"Permit locally-initiated traffic on 10/8")
		.description(L"This filter is part of a rule that permits LAN traffic")
		.provider(MullvadGuids::Provider())
		.layer(FWPM_LAYER_ALE_AUTH_CONNECT_V4)
		.sublayer(MullvadGuids::SublayerWhitelist())
		.weight(wfp::FilterBuilder::WeightClass::Max)
		.permit();

	wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_CONNECT_V4);

	conditionBuilder.add_condition(ConditionIp::Local(wfp::IpAddress::Literal({ 10, 0, 0, 0 }), uint8_t(8)));
	conditionBuilder.add_condition(ConditionIp::Remote(wfp::IpAddress::Literal({ 10, 0, 0, 0 }), uint8_t(8)));

	if (!objectInstaller.addFilter(filterBuilder, conditionBuilder))
	{
		return false;
	}

	//
	// #2 locally-initiated on 172.16/12
	//

	filterBuilder
		.key(MullvadGuids::FilterPermitLan_172_16_12())
		.name(L"Permit locally-initiated traffic on 172.16/12");

	conditionBuilder.reset();

	conditionBuilder.add_condition(ConditionIp::Local(wfp::IpAddress::Literal({ 172, 16, 0, 0 }), uint8_t(12)));
	conditionBuilder.add_condition(ConditionIp::Remote(wfp::IpAddress::Literal({ 172, 16, 0, 0 }), uint8_t(12)));

	if (!objectInstaller.addFilter(filterBuilder, conditionBuilder))
	{
		return false;
	}

	//
	// #3 locally-initiated on 192.168/16
	//

	filterBuilder
		.key(MullvadGuids::FilterPermitLan_192_168_16())
		.name(L"Permit locally-initiated traffic on 192.168/16");

	conditionBuilder.reset();

	conditionBuilder.add_condition(ConditionIp::Local(wfp::IpAddress::Literal({ 192, 168, 0, 0 }), uint8_t(16)));
	conditionBuilder.add_condition(ConditionIp::Remote(wfp::IpAddress::Literal({ 192, 168, 0, 0 }), uint8_t(16)));

	if (!objectInstaller.addFilter(filterBuilder, conditionBuilder))
	{
		return false;
	}

	//
	// #4 LAN to multicast
	//

	filterBuilder
		.key(MullvadGuids::FilterPermitLan_Multicast())
		.name(L"Permit locally-initiated multicast traffic");

	conditionBuilder.reset();

	conditionBuilder.add_condition(ConditionIp::Local(wfp::IpAddress::Literal({ 10, 0, 0, 0 }), uint8_t(8)));
	conditionBuilder.add_condition(ConditionIp::Local(wfp::IpAddress::Literal({ 172, 16, 0, 0 }), uint8_t(12)));
	conditionBuilder.add_condition(ConditionIp::Local(wfp::IpAddress::Literal({ 192, 168, 0, 0 }), uint8_t(16)));
	conditionBuilder.add_condition(ConditionIp::Remote(wfp::IpAddress::Literal({ 224, 0, 0, 0 }), uint8_t(24)));

	return objectInstaller.addFilter(filterBuilder, conditionBuilder);
}

}
