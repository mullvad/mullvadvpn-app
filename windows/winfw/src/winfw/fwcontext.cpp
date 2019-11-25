#include "stdafx.h"
#include "fwcontext.h"
#include "mullvadobjects.h"
#include "objectpurger.h"
#include "rules/blockall.h"
#include "rules/ifirewallrule.h"
#include "rules/permitdhcp.h"
#include "rules/permitndp.h"
#include "rules/permitdhcpserver.h"
#include "rules/permitlan.h"
#include "rules/permitlanservice.h"
#include "rules/permitloopback.h"
#include "rules/permitvpnrelay.h"
#include "rules/permitvpntunnel.h"
#include "rules/permitvpntunnelservice.h"
#include "rules/permitping.h"
#include "rules/restrictdns.h"
#include "libwfp/transaction.h"
#include "libwfp/filterengine.h"
#include <functional>
#include <stdexcept>
#include <utility>

namespace
{

rules::PermitVpnRelay::Protocol TranslateProtocol(WinFwProtocol protocol)
{
	switch (protocol)
	{
		case Tcp: return rules::PermitVpnRelay::Protocol::Tcp;
		case Udp: return rules::PermitVpnRelay::Protocol::Udp;
		default:
		{
			throw std::logic_error("Missing case handler in switch clause");
		}
	};
}

void AppendSettingsRules(FwContext::Ruleset &ruleset, const WinFwSettings &settings)
{
	if (settings.permitDhcp)
	{
		ruleset.emplace_back(std::make_unique<rules::PermitDhcp>());
		ruleset.emplace_back(std::make_unique<rules::PermitNdp>());
	}

	if (settings.permitLan)
	{
		ruleset.emplace_back(std::make_unique<rules::PermitLan>());
		ruleset.emplace_back(std::make_unique<rules::PermitLanService>());
		ruleset.emplace_back(rules::PermitDhcpServer::WithExtent(rules::PermitDhcpServer::Extent::IPv4Only));
	}
}

void AppendNetBlockedRules(FwContext::Ruleset &ruleset)
{
	ruleset.emplace_back(std::make_unique<rules::BlockAll>());
	ruleset.emplace_back(std::make_unique<rules::PermitLoopback>());
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
		throw std::runtime_error("Failed to apply base configuration in BFE.");
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
		throw std::runtime_error("Failed to apply base configuration in BFE.");
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
	AppendSettingsRules(ruleset, settings);

	ruleset.emplace_back(std::make_unique<rules::PermitVpnRelay>(
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
			ruleset.emplace_back(std::make_unique<rules::PermitPing>(
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
	const wchar_t *tunnelInterfaceAlias,
	const wchar_t *v4DnsHost,
	const wchar_t *v6DnsHost
)
{
	Ruleset ruleset;

	AppendNetBlockedRules(ruleset);
	AppendSettingsRules(ruleset, settings);

	ruleset.emplace_back(std::make_unique<rules::PermitVpnRelay>(
		wfp::IpAddress(relay.ip),
		relay.port,
		TranslateProtocol(relay.protocol)
	));

	ruleset.emplace_back(std::make_unique<rules::PermitVpnTunnel>(
		tunnelInterfaceAlias
	));

	ruleset.emplace_back(std::make_unique<rules::PermitVpnTunnelService>(
		tunnelInterfaceAlias
	));

	ruleset.emplace_back(std::make_unique<rules::RestrictDns>(
		tunnelInterfaceAlias,
		wfp::IpAddress(v4DnsHost),
		(v6DnsHost != nullptr) ? std::make_unique<wfp::IpAddress>(v6DnsHost) : nullptr
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
		&& controller.addSublayer(*MullvadObjects::SublayerWhitelist())
		&& controller.addSublayer(*MullvadObjects::SublayerBlacklist());
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
