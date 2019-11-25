#include "stdafx.h"
#include "permitping.h"
#include "winfw/mullvadguids.h"
#include "libwfp/filterbuilder.h"
#include "libwfp/conditionbuilder.h"
#include "libwfp/conditions/conditionip.h"
#include "libwfp/conditions/conditioninterface.h"
#include "libwfp/conditions/conditionprotocol.h"


using namespace wfp::conditions;

namespace rules
{

PermitPing::PermitPing
(
	const std::optional<std::wstring> &interfaceAlias,
	const wfp::IpAddress &host
)
	: m_interfaceAlias(interfaceAlias)
	, m_host(host)
{
}

bool PermitPing::apply(IObjectInstaller &objectInstaller)
{
	if (wfp::IpAddress::Type::Ipv4 == m_host.type())
	{
		return applyIcmpv4(objectInstaller);
	}

	return applyIcmpv6(objectInstaller);
}

bool PermitPing::applyIcmpv4(IObjectInstaller &objectInstaller) const
{
	wfp::FilterBuilder filterBuilder;

	//
	// #1 Permit outbound ICMPv4 to %host% on %interface%
	//

	filterBuilder
		.key(MullvadGuids::FilterPermitPing_Outbound_Icmpv4())
		.name(L"Permit outbound ICMP to specific host (ICMPv4)")
		.description(L"This filter is part of a rule that permits ping")
		.provider(MullvadGuids::Provider())
		.layer(FWPM_LAYER_ALE_AUTH_CONNECT_V4)
		.sublayer(MullvadGuids::SublayerWhitelist())
		.weight(wfp::FilterBuilder::WeightClass::Max)
		.permit();

	wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_CONNECT_V4);

	conditionBuilder.add_condition(ConditionIp::Remote(m_host));
	conditionBuilder.add_condition(ConditionProtocol::Icmp());

	if (m_interfaceAlias.has_value())
	{
		conditionBuilder.add_condition(ConditionInterface::Alias(m_interfaceAlias.value()));
	}

	return objectInstaller.addFilter(filterBuilder, conditionBuilder);
}

bool PermitPing::applyIcmpv6(IObjectInstaller &objectInstaller) const
{
	wfp::FilterBuilder filterBuilder;

	//
	// #1 Permit outbound ICMPv6 to %host% on %interface%
	//

	filterBuilder
		.key(MullvadGuids::FilterPermitPing_Outbound_Icmpv6())
		.name(L"Permit outbound ICMP to specific host (ICMPv6)")
		.description(L"This filter is part of a rule that permits ping")
		.provider(MullvadGuids::Provider())
		.layer(FWPM_LAYER_ALE_AUTH_CONNECT_V6)
		.sublayer(MullvadGuids::SublayerWhitelist())
		.weight(wfp::FilterBuilder::WeightClass::Max)
		.permit();

	wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_CONNECT_V6);

	conditionBuilder.add_condition(ConditionIp::Remote(m_host));
	conditionBuilder.add_condition(ConditionProtocol::IcmpV6());

	if (m_interfaceAlias.has_value())
	{
		conditionBuilder.add_condition(ConditionInterface::Alias(m_interfaceAlias.value()));
	}

	return objectInstaller.addFilter(filterBuilder, conditionBuilder);
}

}
