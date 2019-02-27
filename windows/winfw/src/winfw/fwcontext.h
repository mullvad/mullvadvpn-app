#pragma once

#include "winfw.h"
#include "sessioncontroller.h"
#include "rules/ifirewallrule.h"
#include <cstdint>
#include <memory>
#include <vector>

class FwContext
{
public:

	FwContext(uint32_t timeout);

	bool applyPolicyConnecting(const WinFwSettings &settings, const WinFwRelay &relay);
	bool applyPolicyConnected(const WinFwSettings &settings, const WinFwRelay &relay, const wchar_t *tunnelInterfaceAlias, const wchar_t *v4DnsHosts, const wchar_t *v6DnsHost);
	bool applyPolicyBlocked(const WinFwSettings &settings);

	bool reset();

	using Ruleset = std::vector<std::unique_ptr<rules::IFirewallRule> >;

private:

	FwContext(const FwContext &) = delete;
	FwContext &operator=(const FwContext &) = delete;

	bool applyBaseConfiguration();
	bool applyRuleset(const Ruleset &ruleset);

	std::unique_ptr<SessionController> m_sessionController;

	uint32_t m_baseline;
};
