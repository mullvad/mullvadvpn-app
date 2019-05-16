#include "stdafx.h"
#include "permitlanservice.h"
#include "winfw/mullvadguids.h"
#include "libwfp/filterbuilder.h"
#include "libwfp/conditionbuilder.h"
#include "libwfp/ipaddress.h"
#include "libwfp/conditions/conditionip.h"

using namespace wfp::conditions;

namespace rules
{

bool PermitLanService::apply(IObjectInstaller &objectInstaller)
{
	return applyIpv4(objectInstaller) && applyIpv6(objectInstaller);
}

bool PermitLanService::applyIpv4(IObjectInstaller &objectInstaller) const
{
	wfp::FilterBuilder filterBuilder;

	//
	// #1 incoming request
	//

	filterBuilder
		.key(MullvadGuids::FilterPermitLanService_Inbound_Ipv4())
		.name(L"Permit incoming requests on LAN")
		.description(L"This filter is part of a rule that permits hosting services in a LAN environment")
		.provider(MullvadGuids::Provider())
		.layer(FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V4)
		.sublayer(MullvadGuids::SublayerWhitelist())
		.weight(wfp::FilterBuilder::WeightClass::Max)
		.permit();

	wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V4);

	conditionBuilder.add_condition(ConditionIp::Remote(wfp::IpAddress::Literal({ 10, 0, 0, 0 }), uint8_t(8)));
	conditionBuilder.add_condition(ConditionIp::Remote(wfp::IpAddress::Literal({ 172, 16, 0, 0 }), uint8_t(12)));
	conditionBuilder.add_condition(ConditionIp::Remote(wfp::IpAddress::Literal({ 192, 168, 0, 0 }), uint8_t(16)));
	conditionBuilder.add_condition(ConditionIp::Remote(wfp::IpAddress::Literal({ 169, 254, 0, 0 }), uint8_t(16)));

	return objectInstaller.addFilter(filterBuilder, conditionBuilder);
}

bool PermitLanService::applyIpv6(IObjectInstaller &objectInstaller) const
{
	wfp::FilterBuilder filterBuilder;

	//
	// #1 incoming request
	//

	filterBuilder
		.key(MullvadGuids::FilterPermitLanService_Inbound_Ipv6())
		.name(L"Permit incoming requests on LAN")
		.description(L"This filter is part of a rule that permits hosting services in a LAN environment")
		.provider(MullvadGuids::Provider())
		.layer(FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V6)
		.sublayer(MullvadGuids::SublayerWhitelist())
		.weight(wfp::FilterBuilder::WeightClass::Max)
		.permit();

	wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V6);

	wfp::IpAddress::Literal6 fe80{ 0xFE80, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0 };

	conditionBuilder.add_condition(ConditionIp::Remote(fe80, uint8_t(10)));

	return objectInstaller.addFilter(filterBuilder, conditionBuilder);
}

}
