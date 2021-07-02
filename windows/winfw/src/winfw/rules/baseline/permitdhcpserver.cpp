#include "stdafx.h"
#include "permitdhcpserver.h"
#include <winfw/mullvadguids.h>
#include <winfw/rules/ports.h>
#include <libwfp/filterbuilder.h>
#include <libwfp/conditionbuilder.h>
#include <libwfp/ipaddress.h>
#include <libwfp/conditions/conditionprotocol.h>
#include <libwfp/conditions/conditionport.h>
#include <libwfp/conditions/conditionip.h>
#include <libcommon/error.h>

using namespace wfp::conditions;

namespace rules::baseline
{

//static
std::unique_ptr<PermitDhcpServer> PermitDhcpServer::WithExtent(Extent extent)
{
	if (extent != Extent::IPv4Only)
	{
		THROW_ERROR("The only supported mode is IPv4Only");
	}

	return std::unique_ptr<PermitDhcpServer>(new PermitDhcpServer);
}

bool PermitDhcpServer::apply(IObjectInstaller &objectInstaller)
{
	return applyIpv4(objectInstaller);
}

bool PermitDhcpServer::applyIpv4(IObjectInstaller &objectInstaller) const
{
	//
	// #1 Permit inbound DHCPv4 requests.
	//

	wfp::FilterBuilder filterBuilder;

	filterBuilder
		.key(MullvadGuids::Filter_Baseline_PermitDhcpServer_Inbound_Request_Ipv4())
		.name(L"Permit inbound DHCP requests (IPv4)")
		.description(L"This filter is part of a rule that permits DHCP server traffic")
		.provider(MullvadGuids::Provider())
		.layer(FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V4)
		.sublayer(MullvadGuids::SublayerBaseline())
		.weight(wfp::FilterBuilder::WeightClass::Medium)
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
	// #2 Permit outbound DHCPv4 responses.
	//

	filterBuilder
		.key(MullvadGuids::Filter_Baseline_PermitDhcpServer_Outbound_Response_Ipv4())
		.name(L"Permit outbound DHCP responses (IPv4)")
		.layer(FWPM_LAYER_ALE_AUTH_CONNECT_V4);

	wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_CONNECT_V4);

	conditionBuilder.add_condition(ConditionProtocol::Udp());
	conditionBuilder.add_condition(ConditionPort::Local(DHCPV4_SERVER_PORT));
	conditionBuilder.add_condition(ConditionPort::Remote(DHCPV4_CLIENT_PORT));

	return objectInstaller.addFilter(filterBuilder, conditionBuilder);
}

}
