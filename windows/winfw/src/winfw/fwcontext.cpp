#include "stdafx.h"
#include "fwcontext.h"
#include "mullvadobjects.h"
#include "rules/blockall.h"
#include "rules/ifirewallrule.h"
#include "rules/permitdhcp.h"
#include "rules/permitlan.h"
#include "rules/permitlanservice.h"
#include "rules/permitloopback.h"
#include "rules/permitvpnrelay.h"
#include "rules/permitvpntunnel.h"
#include "rules/permitvpntunnelservice.h"
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
	}

	if (settings.permitLan)
	{
		ruleset.emplace_back(std::make_unique<rules::PermitLan>());
		ruleset.emplace_back(std::make_unique<rules::PermitLanService>());
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

bool FwContext::applyPolicyConnecting(const WinFwSettings &settings, const WinFwRelay &relay)
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

bool FwContext::applyPolicyConnected(const WinFwSettings &settings, const WinFwRelay &relay, const wchar_t *tunnelInterfaceAlias, const wchar_t *v4Gateway, const wchar_t *v6Gateway)
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

	/// We currently expect DNS servers to only be ran on the tunnel gateway IPs
	ruleset.emplace_back(std::make_unique<rules::RestrictDns>(
		tunnelInterfaceAlias,
		wfp::IpAddress(v4Gateway),
		(v6Gateway != nullptr) ? &wfp::IpAddress(v6Gateway) : nullptr
	));

	return applyRuleset(ruleset);
}

bool FwContext::applyPolicyBlocked(const WinFwSettings &settings)
{
	Ruleset ruleset;

	AppendNetBlockedRules(ruleset);
	AppendSettingsRules(ruleset, settings);

	return applyRuleset(ruleset);
}

bool FwContext::reset()
{
	return m_sessionController->executeTransaction([this]()
	{
		m_sessionController->revert(m_baseline);
		return true;
	});
}

bool FwContext::applyRuleset(const Ruleset &ruleset)
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

bool FwContext::applyBaseConfiguration()
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
