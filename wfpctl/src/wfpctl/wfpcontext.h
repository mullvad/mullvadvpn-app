#pragma once

#include "wfpctl.h"
#include "sessioncontroller.h"
#include "rules/ifirewallrule.h"
#include <cstdint>
#include <memory>
#include <vector>

class WfpContext
{
public:

	WfpContext(uint32_t timeout);

	bool applyPolicyConnecting(const WfpctlSettings &settings, const WfpctlRelay &relay);
	bool applyPolicyConnected(const WfpctlSettings &settings, const WfpctlRelay &relay, const wchar_t *tunnelInterfaceAlias, const wchar_t *primaryDns);
	bool applyPolicyNetBlocked();

	bool reset();

	using Ruleset = std::vector<std::unique_ptr<rules::IFirewallRule> >;

private:

	WfpContext(const WfpContext &) = delete;
	WfpContext &operator=(const WfpContext &) = delete;

	bool applyBaseConfiguration();
	bool applyRuleset(const Ruleset &ruleset);

	std::unique_ptr<SessionController> m_sessionController;

	uint32_t m_baseline;
};
