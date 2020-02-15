#include "stdafx.h"
#include "permitselected.h"
#include <winfw/mullvadguids.h>
#include <libwfp/filterbuilder.h>
#include <libwfp/conditionbuilder.h>
#include <libwfp/conditions/conditionport.h>
#include <libwfp/conditions/conditionip.h>
#include <libwfp/conditions/conditioninterface.h>
#include <libcommon/error.h>

using namespace wfp::conditions;

namespace rules::tunneldns
{

namespace
{

static const uint32_t DNS_SERVER_PORT = 53;

} // anonymous namespace

PermitSelected::PermitSelected(const std::wstring &tunnelInterfaceAlias, const std::vector<wfp::IpAddress> &hosts)
	: m_tunnelInterfaceAlias(tunnelInterfaceAlias)
{
	if (hosts.empty())
	{
		THROW_ERROR("Invalid argument: No hosts specified");
	}

	for (const auto &host : hosts)
	{
		switch (host.type())
		{
			case wfp::IpAddress::Type::Ipv4:
			{
				m_hostsIpv4.push_back(host);
				break;
			}
			case wfp::IpAddress::Type::Ipv6:
			{
				m_hostsIpv6.push_back(host);
				break;
			}
			default:
			{
				THROW_ERROR("Missing case handler in switch clause");
			}
		}
	}
}

bool PermitSelected::apply(IObjectInstaller &objectInstaller)
{
	wfp::FilterBuilder filterBuilder;

	//
	// #1 Permit outbound DNS, IPv4.
	//

	if (false == m_hostsIpv4.empty())
	{
		filterBuilder
			.key(MullvadGuids::Filter_TunnelDns_PermitSelected_Outbound_Ipv4())
			.name(L"Permit outbound connections to selected DNS servers (IPv4)")
			.description(L"This filter is part of a rule that permits outbound DNS")
			.provider(MullvadGuids::Provider())
			.layer(FWPM_LAYER_ALE_AUTH_CONNECT_V4)
			.sublayer(MullvadGuids::SublayerTunnelDns())
			.weight(wfp::FilterBuilder::WeightClass::Max)
			.permit();

		wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_CONNECT_V4);

		conditionBuilder.add_condition(ConditionPort::Remote(DNS_SERVER_PORT));
		conditionBuilder.add_condition(ConditionInterface::Alias(m_tunnelInterfaceAlias));

		for (const auto &host : m_hostsIpv4)
		{
			conditionBuilder.add_condition(ConditionIp::Remote(host));
		}

		if (false == objectInstaller.addFilter(filterBuilder, conditionBuilder))
		{
			return false;
		}
	}

	if (m_hostsIpv6.empty())
	{
		return true;
	}

	//
	// #2 Permit outbound DNS, IPv6.
	//

	filterBuilder
		.key(MullvadGuids::Filter_TunnelDns_PermitSelected_Outbound_Ipv6())
		.name(L"Permit outbound connections to selected DNS servers (IPv6)")
		.description(L"This filter is part of a rule that permits outbound DNS")
		.provider(MullvadGuids::Provider())
		.layer(FWPM_LAYER_ALE_AUTH_CONNECT_V6)
		.sublayer(MullvadGuids::SublayerTunnelDns())
		.weight(wfp::FilterBuilder::WeightClass::Max)
		.permit();

	wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_CONNECT_V6);

	conditionBuilder.add_condition(ConditionPort::Remote(DNS_SERVER_PORT));
	conditionBuilder.add_condition(ConditionInterface::Alias(m_tunnelInterfaceAlias));

	for (const auto &host : m_hostsIpv6)
	{
		conditionBuilder.add_condition(ConditionIp::Remote(host));
	}

	return objectInstaller.addFilter(filterBuilder, conditionBuilder);
}

}
