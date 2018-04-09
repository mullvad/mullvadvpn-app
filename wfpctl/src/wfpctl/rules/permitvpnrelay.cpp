#include "stdafx.h"
#include "permitvpnrelay.h"
#include "wfpctl/mullvadguids.h"
#include "libwfp/filterbuilder.h"
#include "libwfp/conditionbuilder.h"
#include "libwfp/conditions/conditionprotocol.h"
#include "libwfp/conditions/conditionip.h"
#include "libwfp/conditions/conditionport.h"

using namespace wfp::conditions;

namespace rules
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
			throw std::logic_error("Missing case handler in switch clause");
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
			throw std::logic_error("Missing case handler in switch clause");
		}
	};
}

} // anonymous namespace

PermitVpnRelay::PermitVpnRelay(const wfp::IpAddress &relay, uint16_t relayPort, Protocol protocol)
	: m_relay(relay)
	, m_relayPort(relayPort)
	, m_protocol(protocol)
{
}

bool PermitVpnRelay::apply(IObjectInstaller &objectInstaller)
{
	wfp::FilterBuilder filterBuilder;

	//
	// #1 permit connecting to relay
	//

	filterBuilder
		.key(MullvadGuids::FilterPermitVpnRelay())
		.name(L"Permit outbound connections to VPN relay")
		.description(L"This filter is part of a rule that permits communication with a VPN relay")
		.provider(MullvadGuids::Provider())
		.layer(LayerFromIp(m_relay))
		.sublayer(MullvadGuids::SublayerWhitelist())
		.weight(wfp::FilterBuilder::WeightClass::Max)
		.permit();

	wfp::ConditionBuilder conditionBuilder(LayerFromIp(m_relay));

	conditionBuilder.add_condition(ConditionIp::Remote(m_relay));
	conditionBuilder.add_condition(ConditionPort::Remote(m_relayPort));
	conditionBuilder.add_condition(CreateProtocolCondition(m_protocol));

	return objectInstaller.addFilter(filterBuilder, conditionBuilder);
}

}
