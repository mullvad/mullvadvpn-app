#include "stdafx.h"
#include "permitdhcp.h"
#include "winfw/mullvadguids.h"
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

	const wfp::IpAddress::Literal6 fe80{ 0xFE80, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0 };

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
		conditionBuilder.add_condition(ConditionPort::Local(68));
		conditionBuilder.add_condition(ConditionIp::Remote(wfp::IpAddress::Literal({ 255, 255, 255, 255 })));
		conditionBuilder.add_condition(ConditionPort::Remote(67));

		if (!objectInstaller.addFilter(filterBuilder, conditionBuilder))
		{
			return false;
		}
	}

	//
	// #2 permit outbound DHCPv6 request
	//

	filterBuilder
		.key(MullvadGuids::FilterPermitDhcpV6_Outbound_Request())
		.name(L"Permit outbound DHCPv6 request")
		.layer(FWPM_LAYER_ALE_AUTH_CONNECT_V6);

	{
		wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_CONNECT_V6);

		const wfp::IpAddress::Literal6 linkLocal{ 0xFF02, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1, 0x2 };
		const wfp::IpAddress::Literal6 siteLocal{ 0xFF05, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1, 0x3 };

		conditionBuilder.add_condition(ConditionProtocol::Udp());
		conditionBuilder.add_condition(ConditionIp::Remote(linkLocal));
		conditionBuilder.add_condition(ConditionIp::Remote(siteLocal));
		conditionBuilder.add_condition(ConditionPort::Remote(547));
		conditionBuilder.add_condition(ConditionIp::Local(fe80, uint8_t(10)));
		conditionBuilder.add_condition(ConditionPort::Local(546));

		if (!objectInstaller.addFilter(filterBuilder, conditionBuilder))
		{
			return false;
		}
	}

	//
	// #3 permit inbound DHCPv4 response
	//

	filterBuilder
		.key(MullvadGuids::FilterPermitDhcpV4_Inbound_Response())
		.name(L"Permit inbound DHCPv4 response")
		.layer(FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V4);

	{
		wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V4);

		conditionBuilder.add_condition(ConditionProtocol::Udp());
		conditionBuilder.add_condition(ConditionPort::Remote(67));
		conditionBuilder.add_condition(ConditionPort::Local(68));

		if (!objectInstaller.addFilter(filterBuilder, conditionBuilder))
		{
			return false;
		}
	}

	//
	// #4 permit inbound DHCPv6 response
	//

	filterBuilder
		.key(MullvadGuids::FilterPermitDhcpV6_Inbound_Response())
		.name(L"Permit inbound DHCPv6 response")
		.layer(FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V6);

	wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V6);

	conditionBuilder.add_condition(ConditionProtocol::Udp());
	conditionBuilder.add_condition(ConditionPort::Remote(547));
	conditionBuilder.add_condition(ConditionIp::Local(fe80, uint8_t(10)));
	conditionBuilder.add_condition(ConditionPort::Local(546));

	return objectInstaller.addFilter(filterBuilder, conditionBuilder);
}

}
