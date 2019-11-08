#pragma once

#include "winfw.h"
#include "sessioncontroller.h"
#include "rules/ifirewallrule.h"
#include "libwfp/ipaddress.h"
#include <cstdint>
#include <memory>
#include <vector>
#include <optional>

class FwContext
{
public:

	FwContext(uint32_t timeout);

	// This ctor applies the "blocked" policy.
	FwContext(uint32_t timeout, const WinFwSettings &settings);

	struct PingableHosts
	{
		std::optional<std::wstring> tunnelInterfaceAlias;
		std::vector<wfp::IpAddress> hosts;
	};

	bool applyPolicyConnecting
	(
		const WinFwSettings &settings,
		const WinFwRelay &relay,
		const std::optional<PingableHosts> &pingableHosts
	);

	bool applyPolicyConnected
	(
		const WinFwSettings &settings,
		const WinFwRelay &relay,
		const wchar_t *tunnelInterfaceAlias,
		const wchar_t *v4DnsHost,
		const wchar_t *v6DnsHost
	);
	bool applyPolicyBlocked(const WinFwSettings &settings);

	bool reset();

	using Ruleset = std::vector<std::unique_ptr<rules::IFirewallRule> >;

private:

	FwContext(const FwContext &) = delete;
	FwContext &operator=(const FwContext &) = delete;

	Ruleset composePolicyBlocked(const WinFwSettings &settings);

	bool applyBaseConfiguration();
	bool applyBlockedBaseConfiguration(const WinFwSettings &settings, uint32_t &checkpoint);
	bool applyCommonBaseConfiguration(SessionController &controller, wfp::FilterEngine &engine);

	bool applyRuleset(const Ruleset &ruleset);
	bool applyRulesetDirectly(const Ruleset &ruleset, SessionController &controller);

	std::unique_ptr<SessionController> m_sessionController;

	uint32_t m_baseline;
};
