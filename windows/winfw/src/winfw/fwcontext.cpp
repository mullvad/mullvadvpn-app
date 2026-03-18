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
#include "rules/dns/blockall.h"
#include "rules/dns/permitloopback.h"
#include "rules/dns/permittunnel.h"
#include "rules/dns/permitnontunnel.h"
#include "rules/multi/permitendpoint.h"
#include <libwfp/transaction.h>
#include <libwfp/filterengine.h>
#include <libcommon/error.h>
#include <functional>
#include <utility>

using namespace rules;

namespace
{

//
// Since the PermitLan rule doesn't specifically address DNS, it will allow DNS requests targeting
// a local resolver to leave the machine. From the local resolver the request will either be
// resolved from cache, or forwarded out onto the Internet.
//
// Therefore, we're unconditionally lifting all DNS traffic out of the baseline sublayer and restricting
// it in the DNS sublayer instead. The PermitDNS rule in the baseline sublayer accomplishes this.
//
// This has implications for the way the relay access is configured. In the regular case there
// is no issue: The PermitEndpoint rule can be installed in the baseline sublayer.
//
// However, if the relay is running on the DNS port (53), it would be blocked unless the DNS
// sublayer permits this traffic. For this reason, whenever the relay is on port 53, the
// PermitEndpoint rule has to be installed to the DNS sublayer instead of the baseline sublayer.
//
void AppendSettingsRules
(
	FwContext::Ruleset &ruleset,
	const WinFwSettings &settings,
	const WinFwSublayerGuids &guids
)
{
	if (settings.permitDhcp)
	{
		ruleset.emplace_back(std::make_unique<baseline::PermitDhcp>(guids.baseline));
		ruleset.emplace_back(std::make_unique<baseline::PermitNdp>(guids.baseline));
	}

	if (settings.permitLan)
	{
		ruleset.emplace_back(std::make_unique<baseline::PermitLan>(guids.baseline));
		ruleset.emplace_back(std::make_unique<baseline::PermitLanService>(guids.baseline));
		ruleset.emplace_back(baseline::PermitDhcpServer::WithExtent(baseline::PermitDhcpServer::Extent::IPv4Only, guids.baseline));
	}

	//
	// DNS management
	//

	ruleset.emplace_back(std::make_unique<baseline::PermitDns>(guids.baseline));
	ruleset.emplace_back(std::make_unique<dns::PermitLoopback>(guids.dns));
	ruleset.emplace_back(std::make_unique<dns::BlockAll>(guids.dns));
}

//
// Refer comment on `AppendSettingsRules`.
//
void AppendRelayRules
(
	FwContext::Ruleset &ruleset,
	const WinFwEndpoint &relay,
	const std::vector<std::wstring> &relayClients,
	const WinFwSublayerGuids &guids
)
{
	const GUID &sublayerKey =
	(
		DNS_SERVER_PORT == relay.port
		? guids.dns
		: guids.baseline
	);

	ruleset.emplace_back(std::make_unique<multi::PermitEndpoint>(
		wfp::IpAddress(relay.ip),
		relay.port,
		relay.protocol,
		relayClients,
		sublayerKey
	));
}

//
// Refer comment on `AppendSettingsRules`.
//
void AppendAllowedEndpointRules
(
	FwContext::Ruleset &ruleset,
	const WinFwAllowedEndpoint &endpoint,
	const WinFwSublayerGuids &guids
)
{
	std::vector<std::wstring> clients;
	clients.reserve(endpoint.numClients);
	for (uint32_t i = 0; i < endpoint.numClients; i++) {
		clients.push_back(endpoint.clients[i]);
	}

	const GUID &sublayerKey =
	(
		DNS_SERVER_PORT == endpoint.endpoint.port
		? guids.dns
		: guids.baseline
	);

	ruleset.emplace_back(std::make_unique<multi::PermitEndpoint>(
		wfp::IpAddress(endpoint.endpoint.ip),
		endpoint.endpoint.port,
		endpoint.endpoint.protocol,
		clients,
		sublayerKey
	));
}

void AppendNetBlockedRules(FwContext::Ruleset &ruleset, const WinFwSublayerGuids &guids)
{
	ruleset.emplace_back(std::make_unique<baseline::BlockAll>(guids.baseline));
	ruleset.emplace_back(std::make_unique<baseline::PermitLoopback>(guids.baseline));
}

} // anonymous namespace

FwContext::FwContext
(
	uint32_t timeout,
	const WinFwSublayerGuids &guids
)
	: m_baseline(0)
	, m_activePolicy(Policy::None)
	, m_objects(guids)
	, m_guids(guids)
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
	const WinFwSublayerGuids &guids,
	const WinFwSettings &settings,
	const std::optional<WinFwAllowedEndpoint> &allowedEndpoint
)
	: m_baseline(0)
	, m_activePolicy(Policy::None)
	, m_objects(guids)
	, m_guids(guids)
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
	const std::vector<WinFwEndpoint> &relays,
	const std::optional<wfp::IpAddress> &exitEndpointIp,
	const std::vector<std::wstring> &relayClients,
	const std::optional<std::wstring> &tunnelInterfaceAlias,
	const std::optional<WinFwAllowedEndpoint> &allowedEndpoint,
	const WinFwAllowedTunnelTraffic &allowedTunnelTraffic
)
{
	Ruleset ruleset;

	AppendNetBlockedRules(ruleset, m_guids);
	AppendSettingsRules(ruleset, settings, m_guids);

	for (const auto &relay : relays)
	{
		AppendRelayRules(ruleset, relay, relayClients, m_guids);
	}

	if (allowedEndpoint.has_value())
	{
		AppendAllowedEndpointRules(ruleset, allowedEndpoint.value(), m_guids);
	}

	if (tunnelInterfaceAlias.has_value())
	{
		switch (allowedTunnelTraffic.type)
		{
			case WinFwAllowedTunnelTrafficType::All:
			{
				ruleset.emplace_back(std::make_unique<baseline::PermitVpnTunnel>(
					m_guids.baseline,
					relayClients,
					*tunnelInterfaceAlias,
					std::nullopt,
					exitEndpointIp
				));
				ruleset.emplace_back(std::make_unique<baseline::PermitVpnTunnelService>(
					m_guids.baseline,
					relayClients,
					*tunnelInterfaceAlias,
					std::nullopt,
					exitEndpointIp
				));
				break;
			}
			case WinFwAllowedTunnelTrafficType::One:
			{
				auto onlyEndpoint = std::make_optional<baseline::PermitVpnTunnel::Endpoints>({
						baseline::PermitVpnTunnel::Endpoint{
						wfp::IpAddress(allowedTunnelTraffic.endpoint1->ip),
						allowedTunnelTraffic.endpoint1->port,
						allowedTunnelTraffic.endpoint1->protocol
						},
						std::nullopt,
				});
				ruleset.emplace_back(std::make_unique<baseline::PermitVpnTunnel>(
					m_guids.baseline,
					relayClients,
					*tunnelInterfaceAlias,
					onlyEndpoint,
					exitEndpointIp
				));
				ruleset.emplace_back(std::make_unique<baseline::PermitVpnTunnelService>(
					m_guids.baseline,
					relayClients,
					*tunnelInterfaceAlias,
					onlyEndpoint,
					exitEndpointIp
				));
				break;
			}
			case WinFwAllowedTunnelTrafficType::Two:
			{
				auto endpoints = std::make_optional<baseline::PermitVpnTunnel::Endpoints>({
						baseline::PermitVpnTunnel::Endpoint{
						wfp::IpAddress(allowedTunnelTraffic.endpoint1->ip),
						allowedTunnelTraffic.endpoint1->port,
						allowedTunnelTraffic.endpoint1->protocol
						},
						std::make_optional<baseline::PermitVpnTunnel::Endpoint>({
								wfp::IpAddress(allowedTunnelTraffic.endpoint2->ip),
								allowedTunnelTraffic.endpoint2->port,
								allowedTunnelTraffic.endpoint2->protocol
								})
				});
				ruleset.emplace_back(std::make_unique<baseline::PermitVpnTunnel>(
							m_guids.baseline,
							relayClients,
							*tunnelInterfaceAlias,
							endpoints,
							exitEndpointIp
							));
				ruleset.emplace_back(std::make_unique<baseline::PermitVpnTunnelService>(
							m_guids.baseline,
							relayClients,
							*tunnelInterfaceAlias,
							endpoints,
							exitEndpointIp
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
	const std::vector<WinFwEndpoint> &relays,
	const std::optional<wfp::IpAddress> &exitEndpointIp,
	const std::vector<std::wstring> &relayClients,
	const std::wstring &tunnelInterfaceAlias,
	const std::vector<wfp::IpAddress> &tunnelDnsServers,
	const std::vector<wfp::IpAddress> &nonTunnelDnsServers
)
{
	Ruleset ruleset;

	AppendNetBlockedRules(ruleset, m_guids);
	AppendSettingsRules(ruleset, settings, m_guids);

	for (const auto &relay : relays)
	{
		AppendRelayRules(ruleset, relay, relayClients, m_guids);
	}

	if (!tunnelDnsServers.empty())
	{
		ruleset.emplace_back(std::make_unique<dns::PermitTunnel>(
			m_guids.dns, tunnelInterfaceAlias, tunnelDnsServers
		));
	}
	if (!nonTunnelDnsServers.empty())
	{
		ruleset.emplace_back(std::make_unique<dns::PermitNonTunnel>(
			m_guids.dns, tunnelInterfaceAlias, nonTunnelDnsServers
		));
	}

	ruleset.emplace_back(std::make_unique<baseline::PermitVpnTunnel>(
		m_guids.baseline,
		relayClients,
		tunnelInterfaceAlias,
		std::nullopt,
		exitEndpointIp
	));

	ruleset.emplace_back(std::make_unique<baseline::PermitVpnTunnelService>(
		m_guids.baseline,
		relayClients,
		tunnelInterfaceAlias,
		std::nullopt,
		exitEndpointIp
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

	AppendNetBlockedRules(ruleset, m_guids);
	AppendSettingsRules(ruleset, settings, m_guids);

	if (allowedEndpoint.has_value())
	{
		AppendAllowedEndpointRules(ruleset, allowedEndpoint.value(), m_guids);
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
		&& controller.addSublayer(*m_objects.sublayerBaseline())
		&& controller.addSublayer(*m_objects.sublayerDns());
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
