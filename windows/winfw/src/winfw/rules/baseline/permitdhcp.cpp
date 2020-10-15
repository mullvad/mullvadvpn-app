#include "stdafx.h"
#include "permitdhcp.h"
#include <winfw/mullvadguids.h>
#include <winfw/rules/ports.h>
#include <libwfp/filterbuilder.h>
#include <libwfp/conditionbuilder.h>
#include <libwfp/ipaddress.h>
#include <libwfp/ipnetwork.h>
#include <libwfp/conditions/conditionprotocol.h>
#include <libwfp/conditions/conditionport.h>
#include <libwfp/conditions/conditionip.h>

using namespace wfp::conditions;

namespace rules::baseline
{

bool PermitDhcp::apply(IObjectInstaller &objectInstaller)
{
	return applyIpv4(objectInstaller) && applyIpv6(objectInstaller);
}

bool PermitDhcp::applyIpv4(IObjectInstaller &objectInstaller) const
{
	//
	// First UDP packet for a unique [remote address, port] tuple is mapped into:
	//
	// outbound: FWPM_LAYER_ALE_AUTH_CONNECT_V{4|6}
	// inbound: FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V{4|6}
	//

	wfp::FilterBuilder filterBuilder;

	//
	// #1 Permit outbound DHCPv4 requests.
	//

	filterBuilder
		.key(MullvadGuids::Filter_Baseline_PermitDhcp_Outbound_Request_Ipv4())
		.name(L"Permit outbound DHCP requests (IPv4)")
		.description(L"This filter is part of a rule that permits DHCP client traffic")
		.provider(MullvadGuids::Provider())
		.layer(FWPM_LAYER_ALE_AUTH_CONNECT_V4)
		.sublayer(MullvadGuids::SublayerBaseline())
		.weight(wfp::FilterBuilder::WeightClass::Medium)
		.permit();

	{
		wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_CONNECT_V4);

		conditionBuilder.add_condition(ConditionProtocol::Udp());
		conditionBuilder.add_condition(ConditionPort::Local(DHCPV4_CLIENT_PORT));
		conditionBuilder.add_condition(ConditionIp::Remote(wfp::IpAddress::Literal({ 255, 255, 255, 255 })));
		conditionBuilder.add_condition(ConditionPort::Remote(DHCPV4_SERVER_PORT));

		if (!objectInstaller.addFilter(filterBuilder, conditionBuilder))
		{
			return false;
		}
	}

	//
	// #2 Permit inbound DHCPv4 responses.
	//

	filterBuilder
		.key(MullvadGuids::Filter_Baseline_PermitDhcp_Inbound_Response_Ipv4())
		.name(L"Permit inbound DHCP responses (IPv4)")
		.layer(FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V4);

	wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V4);

	conditionBuilder.add_condition(ConditionProtocol::Udp());
	conditionBuilder.add_condition(ConditionPort::Local(DHCPV4_CLIENT_PORT));
	conditionBuilder.add_condition(ConditionPort::Remote(DHCPV4_SERVER_PORT));

	return objectInstaller.addFilter(filterBuilder, conditionBuilder);
}

bool PermitDhcp::applyIpv6(IObjectInstaller &objectInstaller) const
{
	const wfp::IpNetwork linkLocal(wfp::IpAddress::Literal6({ 0xFE80, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0 }), 10);

	wfp::FilterBuilder filterBuilder;

	//
	// #1 Permit outbound DHCPv6 requests.
	//

	filterBuilder
		.key(MullvadGuids::Filter_Baseline_PermitDhcp_Outbound_Request_Ipv6())
		.name(L"Permit outbound DHCP requests (IPv6)")
		.description(L"This filter is part of a rule that permits DHCP client traffic")
		.provider(MullvadGuids::Provider())
		.layer(FWPM_LAYER_ALE_AUTH_CONNECT_V6)
		.sublayer(MullvadGuids::SublayerBaseline())
		.weight(wfp::FilterBuilder::WeightClass::Medium)
		.permit();

	{
		wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_CONNECT_V6);

		const wfp::IpAddress::Literal6 linkLocalDhcpMulticast({ 0xFF02, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1, 0x2 });
		const wfp::IpAddress::Literal6 siteLocalDhcpMulticast({ 0xFF05, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1, 0x3 });

		conditionBuilder.add_condition(ConditionProtocol::Udp());
		conditionBuilder.add_condition(ConditionIp::Local(linkLocal));
		conditionBuilder.add_condition(ConditionPort::Local(DHCPV6_CLIENT_PORT));
		conditionBuilder.add_condition(ConditionIp::Remote(linkLocalDhcpMulticast));
		conditionBuilder.add_condition(ConditionIp::Remote(siteLocalDhcpMulticast));
		conditionBuilder.add_condition(ConditionPort::Remote(DHCPV6_SERVER_PORT));

		if (!objectInstaller.addFilter(filterBuilder, conditionBuilder))
		{
			return false;
		}
	}

	//
	// #2 Permit inbound DHCPv6 responses.
	//

	filterBuilder
		.key(MullvadGuids::Filter_Baseline_PermitDhcp_Inbound_Response_Ipv6())
		.name(L"Permit inbound DHCP responses (IPv6)")
		.layer(FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V6);

	wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V6);

	conditionBuilder.add_condition(ConditionProtocol::Udp());
	conditionBuilder.add_condition(ConditionIp::Local(linkLocal));
	conditionBuilder.add_condition(ConditionPort::Local(DHCPV6_CLIENT_PORT));
	conditionBuilder.add_condition(ConditionIp::Remote(linkLocal));
	conditionBuilder.add_condition(ConditionPort::Remote(DHCPV6_SERVER_PORT));

	return objectInstaller.addFilter(filterBuilder, conditionBuilder);
}

}
