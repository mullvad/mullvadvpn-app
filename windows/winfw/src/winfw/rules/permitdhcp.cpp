#include "stdafx.h"
#include "permitdhcp.h"
#include "winfw/mullvadguids.h"
#include "libwfp/filterbuilder.h"
#include "libwfp/conditionbuilder.h"
#include "libwfp/ipaddress.h"
#include "libwfp/ipnetwork.h"
#include "libwfp/conditions/conditionprotocol.h"
#include "libwfp/conditions/conditionport.h"
#include "libwfp/conditions/conditionip.h"

using namespace wfp::conditions;

namespace rules
{

namespace
{

static const uint32_t DHCPV4_CLIENT_PORT = 68;
static const uint32_t DHCPV4_SERVER_PORT = 67;
static const uint32_t DHCPV6_CLIENT_PORT = 546;
static const uint32_t DHCPV6_SERVER_PORT = 547;

} // anonymous namespace

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
	// #1 permit outbound DHCPv4 request
	//

	filterBuilder
		.key(MullvadGuids::FilterPermitDhcpV4_Outbound_Request())
		.name(L"Permit outbound DHCPv4 request")
		.description(L"This filter is part of a rule that permits DHCP client traffic")
		.provider(MullvadGuids::Provider())
		.layer(FWPM_LAYER_ALE_AUTH_CONNECT_V4)
		.sublayer(MullvadGuids::SublayerWhitelist())
		.weight(wfp::FilterBuilder::WeightClass::Max)
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
	// #2 permit inbound DHCPv4 response
	//

	filterBuilder
		.key(MullvadGuids::FilterPermitDhcpV4_Inbound_Response())
		.name(L"Permit inbound DHCPv4 response")
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
	// #1 permit outbound DHCPv6 request
	//

	filterBuilder
		.key(MullvadGuids::FilterPermitDhcpV6_Outbound_Request())
		.name(L"Permit outbound DHCPv6 request")
		.layer(FWPM_LAYER_ALE_AUTH_CONNECT_V6);

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
	// #2 permit inbound DHCPv6 response
	//

	filterBuilder
		.key(MullvadGuids::FilterPermitDhcpV6_Inbound_Response())
		.name(L"Permit inbound DHCPv6 response")
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
