#include "stdafx.h"
#include "permitvpntunnel.h"
#include "winfw/mullvadguids.h"
#include "libwfp/filterbuilder.h"
#include "libwfp/conditionbuilder.h"
#include "libwfp/conditions/conditioninterface.h"
#include "libwfp/conditions/conditionport.h"

using namespace wfp::conditions;

namespace
{

constexpr uint16_t DNS_PORT = 53;

} // anonymous namespace

namespace rules
{

PermitVpnTunnel::PermitVpnTunnel(const std::wstring &tunnelInterfaceAlias)
	: m_tunnelInterfaceAlias(tunnelInterfaceAlias)
{
}

bool PermitVpnTunnel::apply(IObjectInstaller &objectInstaller)
{
	wfp::FilterBuilder filterBuilder;

	//
	// #1 permit locally-initiated traffic on tunnel interface, ipv4
	//    except DNS requests
	//

	filterBuilder
		.key(MullvadGuids::FilterPermitVpnTunnel_Outbound_Ipv4())
		.name(L"Permit outbound on tunnel interface (IPv4)")
		.description(L"This filter is part of a rule that permits communications inside the VPN tunnel")
		.provider(MullvadGuids::Provider())
		.layer(FWPM_LAYER_ALE_AUTH_CONNECT_V4)
		.sublayer(MullvadGuids::SublayerWhitelist())
		.weight(wfp::FilterBuilder::WeightClass::Max)
		.permit();

	{
		wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_CONNECT_V4);

		conditionBuilder.add_condition(ConditionInterface::Alias(m_tunnelInterfaceAlias));
		conditionBuilder.add_condition(ConditionPort::Remote(DNS_PORT, CompareNeq()));

		if (!objectInstaller.addFilter(filterBuilder, conditionBuilder))
		{
			return false;
		}
	}

	//
	// #2 permit locally-initiated traffic on tunnel interface, ipv6
	//    except DNS requests
	//

	filterBuilder
		.key(MullvadGuids::FilterPermitVpnTunnel_Outbound_Ipv6())
		.name(L"Permit outbound on tunnel interface (IPv6)")
		.layer(FWPM_LAYER_ALE_AUTH_CONNECT_V6);

	wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_CONNECT_V6);

	conditionBuilder.add_condition(ConditionInterface::Alias(m_tunnelInterfaceAlias));
	conditionBuilder.add_condition(ConditionPort::Remote(DNS_PORT, CompareNeq()));

	return objectInstaller.addFilter(filterBuilder, conditionBuilder);
}

}
