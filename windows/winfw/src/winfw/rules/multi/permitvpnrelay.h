#pragma once

#include "permitvpnrelaybase.h"
#include <winfw/mullvadguids.h>

namespace rules::multi
{

class PermitVpnRelay : public PermitVpnRelayBase
{
public:

	PermitVpnRelay
	(
		const wfp::IpAddress &relay,
		uint16_t relayPort,
		WinFwProtocol protocol,
		const std::wstring &relayClient,
		Sublayer sublayer
	) : PermitVpnRelayBase(MullvadGuids::Filter_Baseline_PermitVpnRelay(), relay, relayPort, protocol, relayClient, sublayer, std::nullopt)
	{
	}
};

}
