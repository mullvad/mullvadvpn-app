#include "stdafx.h"
#include "permitvpntunnelservice.h"
#include "winfw/mullvadguids.h"
#include "libwfp/filterbuilder.h"
#include "libwfp/conditionbuilder.h"
#include "libwfp/conditions/conditioninterface.h"

using namespace wfp::conditions;

namespace rules
{

PermitVpnTunnelService::PermitVpnTunnelService(const std::wstring &tunnelInterfaceAlias)
	: m_tunnelInterfaceAlias(tunnelInterfaceAlias)
{
}

bool PermitVpnTunnelService::apply(IObjectInstaller &objectInstaller)
{
	wfp::FilterBuilder filterBuilder;

	//
	// #1 incoming request on Ipv4
	//

	filterBuilder
		.key(MullvadGuids::FilterPermitVpnTunnelService_Ipv4())
		.name(L"Permit inbound on tunnel interface (IPv4)")
		.description(L"This filter is part of a rule that permits hosting services that listen on the tunnel interface")
		.provider(MullvadGuids::Provider())
		.layer(FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V4)
		.sublayer(MullvadGuids::SublayerWhitelist())
		.weight(wfp::FilterBuilder::WeightClass::Max)
		.permit();

	wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V4);

	conditionBuilder.add_condition(ConditionInterface::Alias(m_tunnelInterfaceAlias));

	if (!objectInstaller.addFilter(filterBuilder, conditionBuilder))
	{
		return false;
	}

	//
	// #2 incoming request on IPv6
	//

	filterBuilder
		.key(MullvadGuids::FilterPermitVpnTunnelService_Ipv6())
		.name(L"Permit inbound on tunnel interface (IPv6)")
		.layer(FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V6);

	conditionBuilder.reset(FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V6);
	conditionBuilder.add_condition(ConditionInterface::Alias(m_tunnelInterfaceAlias));

	return objectInstaller.addFilter(filterBuilder, conditionBuilder);
}

}
