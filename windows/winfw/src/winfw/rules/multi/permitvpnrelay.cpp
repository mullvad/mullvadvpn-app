#include "stdafx.h"
#include "permitvpnrelay.h"
#include <winfw/mullvadguids.h>
#include <libwfp/filterbuilder.h>
#include <libwfp/conditionbuilder.h>
#include <libwfp/conditions/conditionprotocol.h>
#include <libwfp/conditions/conditionip.h>
#include <libwfp/conditions/conditionport.h>
#include <libwfp/conditions/conditionapplication.h>
#include <libcommon/error.h>

using namespace wfp::conditions;

namespace rules::multi
{

namespace
{

const GUID &LayerFromIp(const wfp::IpAddress &ip)
{
	switch (ip.type())
	{
		case wfp::IpAddress::Type::Ipv4: return FWPM_LAYER_ALE_AUTH_CONNECT_V4;
		case wfp::IpAddress::Type::Ipv6: return FWPM_LAYER_ALE_AUTH_CONNECT_V6;
		default:
		{
			THROW_ERROR("Missing case handler in switch clause");
		}
	};
}

std::unique_ptr<ConditionProtocol> CreateProtocolCondition(PermitVpnRelay::Protocol protocol)
{
	switch (protocol)
	{
		case PermitVpnRelay::Protocol::Tcp: return ConditionProtocol::Tcp();
		case PermitVpnRelay::Protocol::Udp: return ConditionProtocol::Udp();
		default:
		{
			THROW_ERROR("Missing case handler in switch clause");
		}
	};
}

const GUID &TranslateSublayer(PermitVpnRelay::Sublayer sublayer)
{
	switch (sublayer)
	{
		case PermitVpnRelay::Sublayer::Baseline: return MullvadGuids::SublayerBaseline();
		case PermitVpnRelay::Sublayer::Dns: return MullvadGuids::SublayerDns();
		default:
		{
			THROW_ERROR("Missing case handler in switch clause");
		}
	};
}

} // anonymous namespace

PermitVpnRelay::PermitVpnRelay
(
	const wfp::IpAddress &relay,
	uint16_t relayPort,
	Protocol protocol,
	const std::vector<std::wstring> &approvedApplications,
	Sublayer sublayer
)
	: m_relay(relay)
	, m_relayPort(relayPort)
	, m_protocol(protocol)
	, m_approvedApplications(approvedApplications)
	, m_sublayer(sublayer)
{
	if (m_approvedApplications.empty())
	{
		THROW_ERROR("Cannot configure relay access without list of approved applications");
	}
}

bool PermitVpnRelay::apply(IObjectInstaller &objectInstaller)
{
	wfp::FilterBuilder filterBuilder;

	//
	// #1 Permit outbound connections to relay.
	//

	filterBuilder
		.key(MullvadGuids::Filter_Baseline_PermitVpnRelay())
		.name(L"Permit outbound connections to VPN relay")
		.description(L"This filter is part of a rule that permits communication with a VPN relay")
		.provider(MullvadGuids::Provider())
		.layer(LayerFromIp(m_relay))
		.sublayer(TranslateSublayer(m_sublayer))
		.weight(wfp::FilterBuilder::WeightClass::Max)
		.permit();

	wfp::ConditionBuilder conditionBuilder(LayerFromIp(m_relay));

	conditionBuilder.add_condition(ConditionIp::Remote(m_relay));
	conditionBuilder.add_condition(ConditionPort::Remote(m_relayPort));
	conditionBuilder.add_condition(CreateProtocolCondition(m_protocol));

	for (const auto &app : m_approvedApplications)
	{
		conditionBuilder.add_condition(std::make_unique<ConditionApplication>(app));
	}

	return objectInstaller.addFilter(filterBuilder, conditionBuilder);
}

}
