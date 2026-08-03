#pragma once

#include "winfw.h"
#include "sessioncontroller.h"
#include "rules/ifirewallrule.h"
#include "libwfp/ipaddress.h"
#include <cstdint>
#include <memory>
#include <vector>
#include <string>
#include <optional>

class FwContext
{
public:

	//
	// The local endpoint of a socket that is excluded from the firewall, along with the
	// applications that are allowed to use it.
	//
	// This owns its strings, unlike `WinFwAllowedEndpoint`, since the set outlives the
	// call that installs it.
	//
	struct ExcludedSocket
	{
		std::wstring ip;
		uint16_t port;
		WinFwProtocol protocol;
		std::vector<std::wstring> clients;
	};

	FwContext(uint32_t timeout);

	// This ctor applies the "blocked" policy.
	FwContext
	(
		uint32_t timeout,
		const WinFwSettings &settings,
		const std::optional<WinFwAllowedEndpoint> &allowedEndpoint
	);

	bool applyPolicyConnecting
	(
		const WinFwSettings &settings,
		const std::vector<WinFwEndpoint> &relays,
		const std::optional<wfp::IpAddress> &exitEndpointIp,
		const std::vector<std::wstring> &relayClients,
		const std::optional<std::wstring> &tunnelInterfaceAlias,
		const std::optional<WinFwAllowedEndpoint> &allowedEndpoint,
		const WinFwAllowedTunnelTraffic &allowedTunnelTraffic
	);

	bool applyPolicyConnected
	(
		const WinFwSettings &settings,
		const std::vector<WinFwEndpoint> &relays,
		const std::optional<wfp::IpAddress> &exitEndpointIp,
		const std::vector<std::wstring> &relayClients,
		const std::wstring &tunnelInterfaceAlias,
		const std::vector<wfp::IpAddress> &tunnelDnsServers,
		const std::vector<wfp::IpAddress> &nonTunnelDnsServers
	);

	bool applyPolicyBlocked(
		const WinFwSettings &settings,
		const std::optional<WinFwAllowedEndpoint> &allowedEndpoint
	);

	//
	// Replace the set of sockets that are excluded from the firewall.
	//
	// The exceptions are owned by this class and reapplied on top of every policy, so they
	// survive policy changes. They are dropped by `reset`.
	//
	bool setExcludedSockets(std::vector<ExcludedSocket> sockets);

	bool reset();

	enum class Policy
	{
		Connecting,
		Connected,
		Blocked,
		None,
	};

	Policy activePolicy() const;

	using Ruleset = std::vector<std::unique_ptr<rules::IFirewallRule> >;

private:

	FwContext(const FwContext &) = delete;
	FwContext &operator=(const FwContext &) = delete;

	Ruleset composePolicyBlocked(const WinFwSettings &settings, const std::optional<WinFwAllowedEndpoint> &allowedEndpoint);
	Ruleset composeExcludedSocketRules() const;

	bool applyBaseConfiguration();
	bool applyBlockedBaseConfiguration
	(
		const WinFwSettings &settings,
		const std::optional<WinFwAllowedEndpoint> &allowedEndpoint,
		uint32_t &checkpoint,
		uint32_t &policyCheckpoint
	);
	bool applyCommonBaseConfiguration(SessionController &controller, wfp::FilterEngine &engine);

	bool applyRuleset(const Ruleset &ruleset);
	bool applyRulesetDirectly(const Ruleset &ruleset, SessionController &controller);

	std::unique_ptr<SessionController> m_sessionController;

	uint32_t m_baseline;

	//
	// Checkpoint taken after the active policy has been installed, but before the excluded
	// socket rules. Reverting to it removes only the excluded socket rules.
	//
	// This must be updated on every path that installs a policy, or `setExcludedSockets`
	// would rewind too far or not far enough.
	//
	uint32_t m_policyCheckpoint;

	std::vector<ExcludedSocket> m_excludedSockets;

	Policy m_activePolicy;
};
