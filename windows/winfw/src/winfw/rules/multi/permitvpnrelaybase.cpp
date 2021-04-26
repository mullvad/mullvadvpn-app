#include "stdafx.h"
#include "permitvpnrelaybase.h"
#include <winfw/mullvadguids.h>
#include <winfw/winfw.h>
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

std::unique_ptr<ConditionProtocol> CreateProtocolCondition(WinFwProtocol protocol)
{
	switch (protocol)
	{
		case WinFwProtocol::Tcp: return ConditionProtocol::Tcp();
		case WinFwProtocol::Udp: return ConditionProtocol::Udp();
		default:
		{
			THROW_ERROR("Missing case handler in switch clause");
		}
	};
}

const GUID &TranslateSublayer(PermitVpnRelayBase::Sublayer sublayer)
{
	switch (sublayer)
	{
		case PermitVpnRelayBase::Sublayer::Baseline: return MullvadGuids::SublayerBaseline();
		case PermitVpnRelayBase::Sublayer::Dns: return MullvadGuids::SublayerDns();
		default:
		{
			THROW_ERROR("Missing case handler in switch clause");
		}
	};
}

} // anonymous namespace

PermitVpnRelayBase::PermitVpnRelayBase
(
	const GUID &filterKey,
	const wfp::IpAddress &relay,
	uint16_t relayPort,
	WinFwProtocol protocol,
	const std::wstring &relayClient,
	Sublayer sublayer
)
	: m_filterKey(filterKey)
	, m_relay(relay)
	, m_relayPort(relayPort)
	, m_protocol(protocol)
	, m_relayClient(relayClient)
	, m_sublayer(sublayer)
{
}

bool PermitVpnRelayBase::apply(IObjectInstaller &objectInstaller)
{
	wfp::FilterBuilder filterBuilder;

	//
	// #1 Permit outbound connections to relay.
	//

	filterBuilder
		.key(m_filterKey)
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
	conditionBuilder.add_condition(std::make_unique<ConditionApplication>(m_relayClient));

	return objectInstaller.addFilter(filterBuilder, conditionBuilder);
}

}
