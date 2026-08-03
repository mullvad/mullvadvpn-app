#include "stdafx.h"
#include "permitlocalendpoint.h"
#include <winfw/mullvadguids.h>
#include <winfw/winfw.h>
#include <winfw/rules/shared.h>
#include <libwfp/filterbuilder.h>
#include <libwfp/conditionbuilder.h>
#include <libwfp/conditions/conditionprotocol.h>
#include <libwfp/conditions/conditionip.h>
#include <libwfp/conditions/conditionport.h>
#include <libwfp/conditions/conditionapplication.h>
#include <libcommon/error.h>
#include <cstring>

using namespace wfp::conditions;

namespace rules::multi
{

namespace
{

//
// Determine whether `ip` is the unspecified address, i.e. `0.0.0.0` or `::`.
//
// Sockets are commonly bound to the unspecified address, in which case there is no useful
// local IP to filter on.
//
bool IsUnspecified(const wfp::IpAddress &ip)
{
	switch (ip.type())
	{
		case wfp::IpAddress::Type::Ipv4:
		{
			return 0 == ip.addr();
		}
		case wfp::IpAddress::Type::Ipv6:
		{
			static const uint8_t ZEROES[16] = { 0 };
			return 0 == memcmp(ip.addr6().byteArray16, ZEROES, sizeof(ZEROES));
		}
		default:
		{
			THROW_ERROR("Missing case handler in switch clause");
		}
	};
}

} // anonymous namespace

PermitLocalEndpoint::PermitLocalEndpoint
(
	const wfp::IpAddress &localIp,
	uint16_t localPort,
	WinFwProtocol protocol,
	const std::vector<std::wstring> &clients
)
	: m_localIp(localIp)
	, m_localPort(localPort)
	, m_protocol(protocol)
	, m_clients(clients)
{
}

bool PermitLocalEndpoint::apply(IObjectInstaller &objectInstaller)
{
	const bool anyAddress = IsUnspecified(m_localIp);

	std::vector<const GUID *> layers;

	if (anyAddress || wfp::IpAddress::Type::Ipv4 == m_localIp.type())
	{
		layers.push_back(&FWPM_LAYER_ALE_AUTH_CONNECT_V4);
	}

	if (anyAddress || wfp::IpAddress::Type::Ipv6 == m_localIp.type())
	{
		layers.push_back(&FWPM_LAYER_ALE_AUTH_CONNECT_V6);
	}

	const GUID *sublayers[] =
	{
		&MullvadGuids::SublayerBaseline(),
		&MullvadGuids::SublayerDns(),
	};

	for (const auto layer : layers)
	{
		for (const auto sublayer : sublayers)
		{
			//
			// Permit outbound connections from the local endpoint.
			//
			// NOTE: Only outbound connections are permitted. Replies on an already established
			// outbound flow are not reclassified at `ALE_AUTH_RECV_ACCEPT`, so there is no need
			// for a corresponding inbound filter.
			//

			wfp::FilterBuilder filterBuilder(wfp::BuilderValidation::OnlyCritical);

			filterBuilder
				.name(L"Permit outbound connections from a local endpoint")
				.description(L"This filter is part of a rule that excludes a socket from the tunnel")
				.provider(MullvadGuids::Provider())
				.layer(*layer)
				.sublayer(*sublayer)
				.weight(wfp::FilterBuilder::WeightClass::Medium)
				.permit();

			wfp::ConditionBuilder conditionBuilder(*layer);

			conditionBuilder.add_condition(ConditionPort::Local(m_localPort));
			conditionBuilder.add_condition(CreateProtocolCondition(m_protocol));

			//
			// A condition on the unspecified address would never match, so it is left out.
			// The filter is instead narrowed by the application condition(s) below.
			//
			if (false == anyAddress)
			{
				conditionBuilder.add_condition(ConditionIp::Local(m_localIp));
			}

			for (const auto &client : m_clients)
			{
				conditionBuilder.add_condition(std::make_unique<ConditionApplication>(client));
			}

			if (false == objectInstaller.addFilter(filterBuilder, conditionBuilder))
			{
				return false;
			}
		}
	}

	return true;
}

}
