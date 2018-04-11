#include "stdafx.h"
#include "wfpcontext.h"
#include "mullvadobjects.h"
#include "rules/blockall.h"
#include "rules/ifirewallrule.h"
#include "rules/permitdhcp.h"
#include "rules/permitlan.h"
#include "rules/permitlanservice.h"
#include "rules/permitloopback.h"
#include "rules/permitvpnrelay.h"
#include "rules/permitvpntunnel.h"
#include "rules/restrictdns.h"
#include "libwfp/transaction.h"
#include "libwfp/filterengine.h"
#include <functional>
#include <stdexcept>
#include <utility>

namespace
{

rules::PermitVpnRelay::Protocol TranslateProtocol(WfpctlProtocol protocol)
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

void AppendSettingsRules(WfpContext::Ruleset &ruleset, const WfpctlSettings &settings)
{
	if (settings.permitDhcp)
	{
		ruleset.emplace_back(std::make_unique<rules::PermitDhcp>());
	}

	if (settings.permitLan)
	{
		ruleset.emplace_back(std::make_unique<rules::PermitLan>());
		ruleset.emplace_back(std::make_unique<rules::PermitLanService>());
	}
}

void AppendNetBlockedRules(WfpContext::Ruleset &ruleset)
{
	ruleset.emplace_back(std::make_unique<rules::BlockAll>());
	ruleset.emplace_back(std::make_unique<rules::PermitLoopback>());
}

} // anonymous namespace

WfpContext::WfpContext(uint32_t timeout)
	: m_baseline(0)
{
	auto engine = wfp::FilterEngine::DynamicSession(timeout);

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

bool WfpContext::applyPolicyConnecting(const WfpctlSettings &settings, const WfpctlRelay &relay)
{
	Ruleset ruleset;

	AppendNetBlockedRules(ruleset);
	AppendSettingsRules(ruleset, settings);

	ruleset.emplace_back(std::make_unique<rules::PermitVpnRelay>(
		wfp::IpAddress(relay.ip),
		relay.port,
		TranslateProtocol(relay.protocol)
	));

	return applyRuleset(ruleset);
}

bool WfpContext::applyPolicyConnected(const WfpctlSettings &settings, const WfpctlRelay &relay, const wchar_t *tunnelInterfaceAlias, const wchar_t *primaryDns)
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

	ruleset.emplace_back(std::make_unique<rules::RestrictDns>(
		tunnelInterfaceAlias,
		wfp::IpAddress(primaryDns)
	));

	return applyRuleset(ruleset);
}

bool WfpContext::applyPolicyNetBlocked()
{
	Ruleset ruleset;

	AppendNetBlockedRules(ruleset);

	return applyRuleset(ruleset);
}

bool WfpContext::reset()
{
	return m_sessionController->executeTransaction([this]()
	{
		m_sessionController->revert(m_baseline);
		return true;
	});
}

bool WfpContext::applyRuleset(const Ruleset &ruleset)
{
	return m_sessionController->executeTransaction([&]()
	{
		m_sessionController->revert(m_baseline);

		for (const auto &rule : ruleset)
		{
			if (false == rule->apply(*m_sessionController))
			{
				return false;
			}
		}

		return true;
	});
}

bool WfpContext::applyBaseConfiguration()
{
	return m_sessionController->executeTransaction([&]()
	{
		//
		// Install structural objects
		//

		return m_sessionController->addProvider(*MullvadObjects::Provider())
			&& m_sessionController->addSublayer(*MullvadObjects::SublayerWhitelist())
			&& m_sessionController->addSublayer(*MullvadObjects::SublayerBlacklist());
	});
}
