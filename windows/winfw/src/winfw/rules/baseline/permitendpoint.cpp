#include "stdafx.h"
#include "permitendpoint.h"
#include <winfw/mullvadguids.h>
#include <libwfp/filterbuilder.h>
#include <libwfp/conditionbuilder.h>
#include <libwfp/conditions/conditionprotocol.h>
#include <libwfp/conditions/conditionip.h>
#include <libwfp/conditions/conditionport.h>
#include <libwfp/conditions/conditionapplication.h>
#include <libcommon/error.h>

using namespace wfp::conditions;

namespace rules::baseline
{

namespace
{

const GUID &OutboundLayerFromIp(const wfp::IpAddress &ip)
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

} // anonymous namespace

PermitEndpoint::PermitEndpoint
(
	const wfp::IpAddress &address,
	const std::vector<std::wstring> &clients,
	uint16_t port,
	WinFwProtocol protocol
)
	: m_address(address)
	, m_clients(clients)
	, m_port(port)
	, m_protocol(protocol)
{
}

bool PermitEndpoint::apply(IObjectInstaller &objectInstaller)
{
	wfp::FilterBuilder filterBuilder;

	//
	// Permit outbound connections to endpoint.
	//

	filterBuilder
		.key(MullvadGuids::Filter_Baseline_PermitEndpoint())
		.name(L"Permit outbound connections to a given endpoint")
		.description(L"This filter is part of a rule that permits traffic to a specific endpoint")
		.provider(MullvadGuids::Provider())
		.layer(OutboundLayerFromIp(m_address))
		.sublayer(MullvadGuids::SublayerBaseline())
		.weight(wfp::FilterBuilder::WeightClass::Max)
		.permit();

	wfp::ConditionBuilder conditionBuilder(OutboundLayerFromIp(m_address));

	conditionBuilder.add_condition(ConditionIp::Remote(m_address));
	conditionBuilder.add_condition(ConditionPort::Remote(m_port));
	conditionBuilder.add_condition(CreateProtocolCondition(m_protocol));

	for (const auto client : m_clients) {
		conditionBuilder.add_condition(std::make_unique<ConditionApplication>(client));
	}

	return objectInstaller.addFilter(filterBuilder, conditionBuilder);
}

}
