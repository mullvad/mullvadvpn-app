#pragma once

#include "permitvpnrelaybase.h"
#include <winfw/mullvadguids.h>

namespace rules::multi
{

class PermitVpnExitRelay : public PermitVpnRelayBase
{
public:

	PermitVpnExitRelay
	(
		const wfp::IpAddress &relay,
		uint16_t relayPort,
		WinFwProtocol protocol,
		const std::wstring &relayClient,
		Sublayer sublayer
	) : PermitVpnRelayBase(MullvadGuids::Filter_Baseline_PermitVpnExitRelay(), relay, relayPort, protocol, relayClient, sublayer)
	{
	}
};

}
