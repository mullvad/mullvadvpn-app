#include "stdafx.h"
#include "fwcontext.h"
#include "mullvadobjects.h"
#include "objectpurger.h"
#include "rules/ifirewallrule.h"
#include "rules/ports.h"
#include "rules/baseline/blockall.h"
#include "rules/baseline/permitdhcp.h"
#include "rules/baseline/permitndp.h"
#include "rules/baseline/permitdhcpserver.h"
#include "rules/baseline/permitlan.h"
#include "rules/baseline/permitlanservice.h"
#include "rules/baseline/permitloopback.h"
#include "rules/baseline/permitvpntunnel.h"
#include "rules/baseline/permitvpntunnelservice.h"
#include "rules/baseline/permitdns.h"
#include "rules/baseline/permitendpoint.h"
#include "rules/dns/blockall.h"
#include "rules/dns/permitloopback.h"
#include "rules/dns/permittunnel.h"
#include "rules/dns/permitnontunnel.h"
#include "rules/multi/permitvpnrelay.h"
#include <libwfp/transaction.h>
#include <libwfp/filterengine.h>
#include <libcommon/error.h>
#include <functional>
#include <utility>

using namespace rules;

namespace
{

//
// Since the PermitLan rule doesn't specifically address DNS, it will allow DNS requests targetting
// a local resolver to leave the machine. From the local resolver the request will either be
// resolved from cache, or forwarded out onto the Internet.
//
// Therefore, we're unconditionally lifting all DNS traffic out of the baseline sublayer and restricting
// it in the DNS sublayer instead. The PermitDNS rule in the baseline sublayer accomplishes this.
//
// This has implications for the way the relay access is configured. In the regular case there
// is no issue: The PermitVpnRelay rule can be installed in the baseline sublayer.
//
// However, if the relay is running on the DNS port (53), it would be blocked unless the DNS
// sublayer permits this traffic. For this reason, whenever the relay is on port 53, the
// PermitVpnRelay rule has to be installed to the DNS sublayer instead of the baseline sublayer.
//
void AppendSettingsRules
(
	FwContext::Ruleset &ruleset,
	const WinFwSettings &settings
)
{
	if (settings.permitDhcp)
	{
		ruleset.emplace_back(std::make_unique<baseline::PermitDhcp>());
		ruleset.emplace_back(std::make_unique<baseline::PermitNdp>());
	}

	if (settings.permitLan)
	{
		ruleset.emplace_back(std::make_unique<baseline::PermitLan>());
		ruleset.emplace_back(std::make_unique<baseline::PermitLanService>());
		ruleset.emplace_back(baseline::PermitDhcpServer::WithExtent(baseline::PermitDhcpServer::Extent::IPv4Only));
	}

	//
	// DNS management
	//

	ruleset.emplace_back(std::make_unique<baseline::PermitDns>());
	ruleset.emplace_back(std::make_unique<dns::PermitLoopback>());
	ruleset.emplace_back(std::make_unique<dns::BlockAll>());
}

//
// Refer comment on `AppendSettingsRules`.
//
void AppendRelayRules
(
	FwContext::Ruleset &ruleset,
	const WinFwEndpoint &relay,
	const std::wstring &relayClient
)
{
	auto sublayer =
	(
		DNS_SERVER_PORT == relay.port
		? rules::multi::PermitVpnRelay::Sublayer::Dns
		: rules::multi::PermitVpnRelay::Sublayer::Baseline
	);

	ruleset.emplace_back(std::make_unique<multi::PermitVpnRelay>(
		wfp::IpAddress(relay.ip),
		relay.port,
		relay.protocol,
		relayClient,
		sublayer
	));
}

//
// Refer comment on `AppendSettingsRules`.
//
void AppendAllowedEndpointRules
(
	FwContext::Ruleset &ruleset,
	const WinFwAllowedEndpoint &endpoint
)
{
	std::vector<std::wstring> clients;
	clients.reserve(endpoint.numClients);
	for (uint32_t i = 0; i < endpoint.numClients; i++) {
		clients.push_back(endpoint.clients[i]);
	}

	ruleset.emplace_back(std::make_unique<baseline::PermitEndpoint>(
		wfp::IpAddress(endpoint.endpoint.ip),
		clients,
		endpoint.endpoint.port,
		endpoint.endpoint.protocol
	));
}

void AppendNetBlockedRules(FwContext::Ruleset &ruleset)
{
	ruleset.emplace_back(std::make_unique<baseline::BlockAll>());
	ruleset.emplace_back(std::make_unique<baseline::PermitLoopback>());
}

} // anonymous namespace

FwContext::FwContext
(
	uint32_t timeout
)
	: m_baseline(0)
	, m_activePolicy(Policy::None)
{
	auto engine = wfp::FilterEngine::StandardSession(timeout);

	//
	// Pass engine ownership to "session controller"
	//
	m_sessionController = std::make_unique<SessionController>(std::move(engine));

	if (false == applyBaseConfiguration())
	{
		THROW_ERROR("Failed to apply base configuration in BFE");
	}

	m_baseline = m_sessionController->checkpoint();
	m_activePolicy = Policy::None;
}

FwContext::FwContext
(
	uint32_t timeout,
	const WinFwSettings &settings,
	const std::optional<WinFwAllowedEndpoint> &allowedEndpoint
)
	: m_baseline(0)
	, m_activePolicy(Policy::None)
{
	auto engine = wfp::FilterEngine::StandardSession(timeout);

	//
	// Pass engine ownership to "session controller"
	//
	m_sessionController = std::make_unique<SessionController>(std::move(engine));

	uint32_t checkpoint = 0;

	if (false == applyBlockedBaseConfiguration(settings, allowedEndpoint, checkpoint))
	{
		THROW_ERROR("Failed to apply base configuration in BFE");
	}

	m_baseline = checkpoint;
	m_activePolicy = Policy::Blocked;
}

bool FwContext::applyPolicyConnecting
(
	const WinFwSettings &settings,
	const WinFwEndpoint &relay,
	const std::wstring &relayClient,
	const std::optional<std::wstring> &tunnelInterfaceAlias,
	const std::optional<WinFwAllowedEndpoint> &allowedEndpoint,
	const WinFwAllowedTunnelTraffic &allowedTunnelTraffic
)
{
	Ruleset ruleset;

	AppendNetBlockedRules(ruleset);
	AppendSettingsRules(ruleset, settings);
	AppendRelayRules(ruleset, relay, relayClient);

	if (allowedEndpoint.has_value())
	{
		AppendAllowedEndpointRules(ruleset, allowedEndpoint.value());
	}

	if (tunnelInterfaceAlias.has_value())
	{
		switch (allowedTunnelTraffic.type)
		{
			case WinFwAllowedTunnelTrafficType::All:
			{
				ruleset.emplace_back(std::make_unique<baseline::PermitVpnTunnel>(
					*tunnelInterfaceAlias,
					std::nullopt
				));
				ruleset.emplace_back(std::make_unique<baseline::PermitVpnTunnelService>(
					*tunnelInterfaceAlias,
					std::nullopt
				));
				break;
			}
			case WinFwAllowedTunnelTrafficType::Only:
			{
				const auto onlyEndpoint = std::make_optional(baseline::PermitVpnTunnel::Endpoint{
					wfp::IpAddress(allowedTunnelTraffic.endpoint->ip),
					allowedTunnelTraffic.endpoint->port,
					allowedTunnelTraffic.endpoint->protocol
				});
				ruleset.emplace_back(std::make_unique<baseline::PermitVpnTunnel>(
					*tunnelInterfaceAlias,
					onlyEndpoint
				));
				ruleset.emplace_back(std::make_unique<baseline::PermitVpnTunnelService>(
					*tunnelInterfaceAlias,
					onlyEndpoint
				));
				break;
			}
			// For the "None" case, do nothing.
		}
	}

	const auto status = applyRuleset(ruleset);

	if (status)
	{
		m_activePolicy = Policy::Connecting;
	}

	return status;
}

bool FwContext::applyPolicyConnected
(
	const WinFwSettings &settings,
	const WinFwEndpoint &relay,
	const std::wstring &relayClient,
	const std::wstring &tunnelInterfaceAlias,
	const std::vector<wfp::IpAddress> &tunnelDnsServers,
	const std::vector<wfp::IpAddress> &nonTunnelDnsServers
)
{
	Ruleset ruleset;

	AppendNetBlockedRules(ruleset);
	AppendSettingsRules(ruleset, settings);
	AppendRelayRules(ruleset, relay, relayClient);

	if (!tunnelDnsServers.empty())
	{
		ruleset.emplace_back(std::make_unique<dns::PermitTunnel>(
			tunnelInterfaceAlias, tunnelDnsServers
		));
	}
	if (!nonTunnelDnsServers.empty())
	{
		ruleset.emplace_back(std::make_unique<dns::PermitNonTunnel>(
			tunnelInterfaceAlias, nonTunnelDnsServers
		));
	}

	ruleset.emplace_back(std::make_unique<baseline::PermitVpnTunnel>(
		tunnelInterfaceAlias,
		std::nullopt
	));

	ruleset.emplace_back(std::make_unique<baseline::PermitVpnTunnelService>(
		tunnelInterfaceAlias,
		std::nullopt
	));

	const auto status = applyRuleset(ruleset);

	if (status)
	{
		m_activePolicy = Policy::Connected;
	}

	return status;
}

bool FwContext::applyPolicyBlocked(const WinFwSettings &settings, const std::optional<WinFwAllowedEndpoint> &allowedEndpoint)
{
	const auto status = applyRuleset(composePolicyBlocked(settings, allowedEndpoint));

	if (status)
	{
		m_activePolicy = Policy::Blocked;
	}

	return status;
}

bool FwContext::reset()
{
	const auto status = m_sessionController->executeTransaction([this](SessionController &controller, wfp::FilterEngine &)
	{
		return controller.revert(m_baseline), true;
	});

	if (status)
	{
		m_activePolicy = Policy::None;
	}

	return status;
}

FwContext::Policy FwContext::activePolicy() const
{
	return m_activePolicy;
}

FwContext::Ruleset FwContext::composePolicyBlocked(const WinFwSettings &settings, const std::optional<WinFwAllowedEndpoint> &allowedEndpoint)
{
	Ruleset ruleset;

	AppendNetBlockedRules(ruleset);
	AppendSettingsRules(ruleset, settings);

	if (allowedEndpoint.has_value())
	{
		AppendAllowedEndpointRules(ruleset, allowedEndpoint.value());
	}

	return ruleset;
}

bool FwContext::applyBaseConfiguration()
{
	return m_sessionController->executeTransaction([this](SessionController &controller, wfp::FilterEngine &engine)
	{
		return applyCommonBaseConfiguration(controller, engine);
	});
}

bool FwContext::applyBlockedBaseConfiguration(const WinFwSettings &settings, const std::optional<WinFwAllowedEndpoint> &allowedEndpoint, uint32_t &checkpoint)
{
	return m_sessionController->executeTransaction([&](SessionController &controller, wfp::FilterEngine &engine)
	{
		if (false == applyCommonBaseConfiguration(controller, engine))
		{
			return false;
		}

		//
		// Record the current session state with only structural objects added.
		// If we snapshot at a later time we'd accidentally include the blocking policy rules
		// in the baseline checkpoint.
		//
		checkpoint = controller.peekCheckpoint();

		return applyRulesetDirectly(composePolicyBlocked(settings, allowedEndpoint), controller);
	});
}

bool FwContext::applyCommonBaseConfiguration(SessionController &controller, wfp::FilterEngine &engine)
{
	//
	// Since we're using a standard WFP session we can make no assumptions
	// about which objects are already installed since before.
	//
	ObjectPurger::GetRemoveAllFunctor()(engine);

	//
	// Install structural objects
	//
	return controller.addProvider(*MullvadObjects::Provider())
		&& controller.addSublayer(*MullvadObjects::SublayerBaseline())
		&& controller.addSublayer(*MullvadObjects::SublayerDns());
}

bool FwContext::applyRuleset(const Ruleset &ruleset)
{
	return m_sessionController->executeTransaction([&](SessionController &controller, wfp::FilterEngine &)
	{
		controller.revert(m_baseline);
		return applyRulesetDirectly(ruleset, controller);
	});
}

bool FwContext::applyRulesetDirectly(const Ruleset &ruleset, SessionController &controller)
{
	for (const auto &rule : ruleset)
	{
		if (false == rule->apply(controller))
		{
			return false;
		}
	}

	return true;
}
