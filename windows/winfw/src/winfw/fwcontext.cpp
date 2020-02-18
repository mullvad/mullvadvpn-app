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
#include "rules/baseline/permitvpnrelay.h"
#include "rules/baseline/permitvpntunnel.h"
#include "rules/baseline/permitvpntunnelservice.h"
#include "rules/baseline/permitping.h"
#include "rules/baseline/permitdns.h"
#include "rules/dns/blockall.h"
#include "rules/dns/permitnontunnel.h"
#include "rules/dns/permittunnel.h"
#include <libwfp/transaction.h>
#include <libwfp/filterengine.h>
#include <libcommon/error.h>
#include <functional>
#include <utility>

using namespace rules;

namespace
{

baseline::PermitVpnRelay::Protocol TranslateProtocol(WinFwProtocol protocol)
{
	switch (protocol)
	{
		case Tcp: return baseline::PermitVpnRelay::Protocol::Tcp;
		case Udp: return baseline::PermitVpnRelay::Protocol::Udp;
		default:
		{
			THROW_ERROR("Missing case handler in switch clause");
		}
	};
}

//
// Since the PermitLan rule doesn't specifically address DNS, it will allow DNS requests targetting
// a local resolver to leave the machine. From the local resolver the request will either be
// resolved from cache, or forwarded out onto the Internet.
//
// Therefore, whenever the PermitLan rule might be activated, we must also do proper DNS management
// to prevent leaks.
//
void AppendSettingsRules
(
	FwContext::Ruleset &ruleset,
	const WinFwSettings &settings,
	std::optional<std::wstring> tunnelInterfaceAlias = std::nullopt,
	std::optional<std::vector<wfp::IpAddress> > nonTunnelDnsServers = std::nullopt,
	std::optional<std::vector<wfp::IpAddress> > tunnelDnsServers = std::nullopt
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
	ruleset.emplace_back(std::make_unique<dns::BlockAll>());

	if (nonTunnelDnsServers.has_value())
	{
		ruleset.emplace_back(std::make_unique<dns::PermitNonTunnel>(
			tunnelInterfaceAlias, nonTunnelDnsServers.value()));
	}

	if (tunnelInterfaceAlias.has_value() && tunnelDnsServers.has_value())
	{
		ruleset.emplace_back(std::make_unique<dns::PermitTunnel>(
			tunnelInterfaceAlias.value(), tunnelDnsServers.value()));
	}
}

void AppendNetBlockedRules(FwContext::Ruleset &ruleset)
{
	ruleset.emplace_back(std::make_unique<baseline::BlockAll>());
	ruleset.emplace_back(std::make_unique<baseline::PermitLoopback>());
}

std::optional<std::vector<wfp::IpAddress> >
CreateRelayDnsExclusion(const WinFwRelay &relay)
{
	if (relay.port != DNS_SERVER_PORT)
	{
		return std::nullopt;
	}

	std::vector<wfp::IpAddress> result = { wfp::IpAddress(relay.ip) };

	return std::move(result);
}

} // anonymous namespace

FwContext::FwContext(uint32_t timeout)
	: m_baseline(0)
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
}

FwContext::FwContext(uint32_t timeout, const WinFwSettings &settings)
	: m_baseline(0)
{
	auto engine = wfp::FilterEngine::StandardSession(timeout);

	//
	// Pass engine ownership to "session controller"
	//
	m_sessionController = std::make_unique<SessionController>(std::move(engine));

	uint32_t checkpoint = 0;

	if (false == applyBlockedBaseConfiguration(settings, checkpoint))
	{
		THROW_ERROR("Failed to apply base configuration in BFE");
	}

	m_baseline = checkpoint;
}

bool FwContext::applyPolicyConnecting
(
	const WinFwSettings &settings,
	const WinFwRelay &relay,
	const std::optional<PingableHosts> &pingableHosts
)
{
	Ruleset ruleset;

	AppendNetBlockedRules(ruleset);
	AppendSettingsRules(ruleset, settings, std::nullopt, CreateRelayDnsExclusion(relay));

	ruleset.emplace_back(std::make_unique<baseline::PermitVpnRelay>(
		wfp::IpAddress(relay.ip),
		relay.port,
		TranslateProtocol(relay.protocol)
	));

	//
	// Permit pinging the gateway inside the tunnel.
	//
	if (pingableHosts.has_value())
	{
		const auto &ph = pingableHosts.value();

		for (const auto &host : ph.hosts)
		{
			ruleset.emplace_back(std::make_unique<baseline::PermitPing>(
				ph.tunnelInterfaceAlias,
				host
			));
		}
	}

	return applyRuleset(ruleset);
}

bool FwContext::applyPolicyConnected
(
	const WinFwSettings &settings,
	const WinFwRelay &relay,
	const std::wstring &tunnelInterfaceAlias,
	const std::vector<wfp::IpAddress> &tunnelDnsServers
)
{
	Ruleset ruleset;

	AppendNetBlockedRules(ruleset);

	AppendSettingsRules
	(
		ruleset,
		settings,
		std::make_optional<>(tunnelInterfaceAlias),
		CreateRelayDnsExclusion(relay),
		std::make_optional<>(tunnelDnsServers)
	);

	ruleset.emplace_back(std::make_unique<baseline::PermitVpnRelay>(
		wfp::IpAddress(relay.ip),
		relay.port,
		TranslateProtocol(relay.protocol)
	));

	ruleset.emplace_back(std::make_unique<baseline::PermitVpnTunnel>(
		tunnelInterfaceAlias
	));

	ruleset.emplace_back(std::make_unique<baseline::PermitVpnTunnelService>(
		tunnelInterfaceAlias
	));

	return applyRuleset(ruleset);
}

bool FwContext::applyPolicyBlocked(const WinFwSettings &settings)
{
	return applyRuleset(composePolicyBlocked(settings));
}

bool FwContext::reset()
{
	return m_sessionController->executeTransaction([this](SessionController &controller, wfp::FilterEngine &)
	{
		return controller.revert(m_baseline), true;
	});
}

FwContext::Ruleset FwContext::composePolicyBlocked(const WinFwSettings &settings)
{
	Ruleset ruleset;

	AppendNetBlockedRules(ruleset);
	AppendSettingsRules(ruleset, settings);

	return ruleset;
}

bool FwContext::applyBaseConfiguration()
{
	return m_sessionController->executeTransaction([this](SessionController &controller, wfp::FilterEngine &engine)
	{
		return applyCommonBaseConfiguration(controller, engine);
	});
}

bool FwContext::applyBlockedBaseConfiguration(const WinFwSettings &settings, uint32_t &checkpoint)
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

		return applyRulesetDirectly(composePolicyBlocked(settings), controller);
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
