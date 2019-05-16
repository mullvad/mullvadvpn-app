#include "stdafx.h"
#include "permitdhcpserver.h"
#include "winfw/mullvadguids.h"
#include "libwfp/filterbuilder.h"
#include "libwfp/conditionbuilder.h"
#include "libwfp/ipaddress.h"
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

} // anonymous namespace

bool PermitDhcpServer::apply(IObjectInstaller &objectInstaller)
{
	return applyIpv4(objectInstaller);
}

bool PermitDhcpServer::applyIpv4(IObjectInstaller &objectInstaller) const
{
	//
	// #1 permit incoming DHCPv4 request
	//

	wfp::FilterBuilder filterBuilder;

	filterBuilder
		.key(MullvadGuids::FilterPermitDhcpV4Server_Inbound_Request())
		.name(L"Permit inbound DHCPv4 request")
		.description(L"This filter is part of a rule that permits DHCP server traffic")
		.provider(MullvadGuids::Provider())
		.layer(FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V4)
		.sublayer(MullvadGuids::SublayerWhitelist())
		.weight(wfp::FilterBuilder::WeightClass::Max)
		.permit();

	{
		wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V4);

		conditionBuilder.add_condition(ConditionProtocol::Udp());
		conditionBuilder.add_condition(ConditionPort::Local(DHCPV4_SERVER_PORT));
		conditionBuilder.add_condition(ConditionIp::Local(wfp::IpAddress::Literal({ 255, 255, 255, 255 })));
		conditionBuilder.add_condition(ConditionPort::Remote(DHCPV4_CLIENT_PORT));

		if (!objectInstaller.addFilter(filterBuilder, conditionBuilder))
		{
			return false;
		}
	}

	//
	// #2 permit outbound DHCPv4 response
	//

	filterBuilder
		.key(MullvadGuids::FilterPermitDhcpV4Server_Outbound_Response())
		.name(L"Permit outbound DHCPv4 response")
		.layer(FWPM_LAYER_ALE_AUTH_CONNECT_V4);

	wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_CONNECT_V4);

	conditionBuilder.add_condition(ConditionProtocol::Udp());
	conditionBuilder.add_condition(ConditionPort::Local(DHCPV4_SERVER_PORT));
	conditionBuilder.add_condition(ConditionPort::Remote(DHCPV4_CLIENT_PORT));

	return objectInstaller.addFilter(filterBuilder, conditionBuilder);
}

}
