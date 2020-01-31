#include "stdafx.h"
#include "permitlan.h"
#include <winfw/mullvadguids.h>
#include <libwfp/filterbuilder.h>
#include <libwfp/conditionbuilder.h>
#include <libwfp/ipaddress.h>
#include <libwfp/ipnetwork.h>
#include <libwfp/conditions/conditionip.h>

using namespace wfp::conditions;

namespace rules::baseline
{

bool PermitLan::apply(IObjectInstaller &objectInstaller)
{
	return applyIpv4(objectInstaller) && applyIpv6(objectInstaller);
}

bool PermitLan::applyIpv4(IObjectInstaller &objectInstaller) const
{
	wfp::FilterBuilder filterBuilder;

	//
	// #1 Permit outbound connections on LAN.
	//

	filterBuilder
		.key(MullvadGuids::Filter_Baseline_PermitLan_Outbound_Ipv4())
		.name(L"Permit outbound connections on LAN (IPv4)")
		.description(L"This filter is part of a rule that permits LAN traffic")
		.provider(MullvadGuids::Provider())
		.layer(FWPM_LAYER_ALE_AUTH_CONNECT_V4)
		.sublayer(MullvadGuids::SublayerBaseline())
		.weight(wfp::FilterBuilder::WeightClass::Max)
		.permit();

	wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_CONNECT_V4);

	conditionBuilder.add_condition(ConditionIp::Remote(wfp::IpNetwork(wfp::IpAddress::Literal({ 10, 0, 0, 0 }), 8)));
	conditionBuilder.add_condition(ConditionIp::Remote(wfp::IpNetwork(wfp::IpAddress::Literal({ 172, 16, 0, 0 }), 12)));
	conditionBuilder.add_condition(ConditionIp::Remote(wfp::IpNetwork(wfp::IpAddress::Literal({ 192, 168, 0, 0 }), 16)));
	conditionBuilder.add_condition(ConditionIp::Remote(wfp::IpNetwork(wfp::IpAddress::Literal({ 169, 254, 0, 0 }), 16)));

	if (!objectInstaller.addFilter(filterBuilder, conditionBuilder))
	{
		return false;
	}

	//
	// #2 Permit outbound multicast on LAN.
	//

	filterBuilder
		.key(MullvadGuids::Filter_Baseline_PermitLan_Outbound_Multicast_Ipv4())
		.name(L"Permit outbound multicast on LAN (IPv4)");

	conditionBuilder.reset();

	// Local subnet multicast.
	conditionBuilder.add_condition(ConditionIp::Remote(wfp::IpNetwork(wfp::IpAddress::Literal({ 224, 0, 0, 0 }), 24)));

	// Simple Service Discovery Protocol (SSDP) address.
	conditionBuilder.add_condition(ConditionIp::Remote(wfp::IpNetwork(wfp::IpAddress::Literal({ 239, 255, 255, 250 }), 32)));

	// mDNS Service Discovery address.
	conditionBuilder.add_condition(ConditionIp::Remote(wfp::IpNetwork(wfp::IpAddress::Literal({ 239, 255, 255, 251 }), 32)));

	return objectInstaller.addFilter(filterBuilder, conditionBuilder);
}

bool PermitLan::applyIpv6(IObjectInstaller &objectInstaller) const
{
	wfp::FilterBuilder filterBuilder;

	//
	// #1 Permit outbound connections on LAN.
	//

	filterBuilder
		.key(MullvadGuids::Filter_Baseline_PermitLan_Outbound_Ipv6())
		.name(L"Permit outbound connections on LAN (IPv6)")
		.description(L"This filter is part of a rule that permits LAN traffic")
		.provider(MullvadGuids::Provider())
		.layer(FWPM_LAYER_ALE_AUTH_CONNECT_V6)
		.sublayer(MullvadGuids::SublayerBaseline())
		.weight(wfp::FilterBuilder::WeightClass::Max)
		.permit();

	wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_CONNECT_V6);

	const wfp::IpNetwork linkLocal(wfp::IpAddress::Literal6({ 0xFE80, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0 }), 10);
	const wfp::IpNetwork uniqueLocal(wfp::IpAddress::Literal6({ 0xFD00, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0 }), 8);

	conditionBuilder.add_condition(ConditionIp::Remote(linkLocal));
	conditionBuilder.add_condition(ConditionIp::Remote(uniqueLocal));

	if (!objectInstaller.addFilter(filterBuilder, conditionBuilder))
	{
		return false;
	}

	//
	// #2 Permit outbound multicast on LAN.
	//

	filterBuilder
		.key(MullvadGuids::Filter_Baseline_PermitLan_Outbound_Multicast_Ipv6())
		.name(L"Permit outbound multicast on LAN (IPv6)");

	conditionBuilder.reset();

	const wfp::IpNetwork linkLocalMulticast(wfp::IpAddress::Literal6({ 0xFF02, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0 }), 16);
	const wfp::IpNetwork siteLocalMulticast(wfp::IpAddress::Literal6({ 0xFF05, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0 }), 16);

	conditionBuilder.add_condition(ConditionIp::Remote(linkLocalMulticast));
	conditionBuilder.add_condition(ConditionIp::Remote(siteLocalMulticast));

	return objectInstaller.addFilter(filterBuilder, conditionBuilder);
}

}
