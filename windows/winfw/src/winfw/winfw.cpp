#include "stdafx.h"
#include "winfw.h"
#include "fwcontext.h"
#include "objectpurger.h"
#include "mullvadobjects.h"
#include "rules/persistent/blockall.h"
#include "libwfp/ipnetwork.h"
#include <windows.h>
#include <libcommon/error.h>
#include <libcommon/string.h>
#include <optional>

namespace
{

constexpr uint32_t DEINITIALIZE_TIMEOUT = 5000;

MullvadLogSink g_logSink = nullptr;
void *g_logSinkContext = nullptr;

FwContext *g_fwContext = nullptr;

std::optional<FwContext::PingableHosts> ConvertPingableHosts(const PingableHosts *pingableHosts)
{
	if (nullptr == pingableHosts)
	{
		return {};
	}

	if (nullptr == pingableHosts->hosts
		|| 0 == pingableHosts->numHosts)
	{
		THROW_ERROR("Invalid PingableHosts structure");
	}

	FwContext::PingableHosts converted;

	if (nullptr != pingableHosts->tunnelInterfaceAlias)
	{
		converted.tunnelInterfaceAlias = pingableHosts->tunnelInterfaceAlias;
	}

	for (size_t i = 0; i < pingableHosts->numHosts; ++i)
	{
		converted.hosts.emplace_back(wfp::IpAddress(pingableHosts->hosts[i]));
	}

	return converted;
}

WINFW_POLICY_STATUS
HandlePolicyException(const common::error::WindowsException &err)
{
	if (nullptr != g_logSink)
	{
		g_logSink(MULLVAD_LOG_LEVEL_ERROR, err.what(), g_logSinkContext);
	}

	if (FWP_E_TIMEOUT == err.errorCode())
	{
		// TODO: Detect software that may cause this
		return WINFW_POLICY_STATUS_LOCK_TIMEOUT;
	}

	return WINFW_POLICY_STATUS_GENERAL_FAILURE;
}

//
// Networks for which DNS requests can be made on all network adapters.
//
// This should be synchronized with `ALLOWED_LAN_NETS` in talpid-core,
// but it also includes loopback addresses.
//
wfp::IpNetwork g_privateIpRanges[] = {
	wfp::IpNetwork(wfp::IpAddress::Literal{127, 0, 0, 0}, 8),
	wfp::IpNetwork(wfp::IpAddress::Literal{10, 0, 0, 0}, 8),
	wfp::IpNetwork(wfp::IpAddress::Literal{172, 16, 0, 0}, 12),
	wfp::IpNetwork(wfp::IpAddress::Literal{192, 168, 0, 0}, 16),
	wfp::IpNetwork(wfp::IpAddress::Literal{169, 254, 0, 0}, 16),
	wfp::IpNetwork(wfp::IpAddress::Literal6{0, 0, 0, 0, 0, 0, 0, 1}, 128),
	wfp::IpNetwork(wfp::IpAddress::Literal6{0xfe80, 0, 0, 0, 0, 0, 0, 0}, 10),
	wfp::IpNetwork(wfp::IpAddress::Literal6{0xfc80, 0, 0, 0, 0, 0, 0, 0}, 7)
};

} // anonymous namespace

WINFW_LINKAGE
bool
WINFW_API
WinFw_Initialize(
	uint32_t timeout,
	MullvadLogSink logSink,
	void *logSinkContext
)
{
	try
	{
		if (nullptr != g_fwContext)
		{
			//
			// This is an error.
			// The existing instance may have a different timeout etc.
			//
			THROW_ERROR("Cannot initialize WINFW twice");
		}

		// Convert seconds to milliseconds.
		uint32_t timeout_ms = timeout * 1000;

		g_logSink = logSink;
		g_logSinkContext = logSinkContext;

		g_fwContext = new FwContext(timeout_ms);
	}
	catch (std::exception &err)
	{
		if (nullptr != g_logSink)
		{
			g_logSink(MULLVAD_LOG_LEVEL_ERROR, err.what(), g_logSinkContext);
		}

		return false;
	}
	catch (...)
	{
		return false;
	}

	return true;
}

extern "C"
WINFW_LINKAGE
bool
WINFW_API
WinFw_InitializeBlocked(
	uint32_t timeout,
	const WinFwSettings *settings,
	MullvadLogSink logSink,
	void *logSinkContext
)
{
	try
	{
		if (nullptr != g_fwContext)
		{
			//
			// This is an error.
			// The existing instance may have a different timeout etc.
			//
			THROW_ERROR("Cannot initialize WINFW twice");
		}

		if (nullptr == settings)
		{
			THROW_ERROR("Invalid argument: settings");
		}

		// Convert seconds to milliseconds.
		uint32_t timeout_ms = timeout * 1000;

		g_logSink = logSink;
		g_logSinkContext = logSinkContext;

		g_fwContext = new FwContext(timeout_ms, *settings);
	}
	catch (std::exception &err)
	{
		if (nullptr != g_logSink)
		{
			g_logSink(MULLVAD_LOG_LEVEL_ERROR, err.what(), g_logSinkContext);
		}

		return false;
	}
	catch (...)
	{
		return false;
	}

	return true;
}

WINFW_LINKAGE
bool
WINFW_API
WinFw_Deinitialize(WINFW_CLEANUP_POLICY cleanupPolicy)
{
	if (nullptr == g_fwContext)
	{
		return true;
	}

	const auto activePolicy = g_fwContext->activePolicy();

	//
	// Do not use FwContext::reset() here because it just
	// removes the current policy but leaves sublayers etc.
	//

	delete g_fwContext;
	g_fwContext = nullptr;

	//
	// Continue blocking if this is what the caller requested
	// and if the current policy is "(net) blocked".
	//

	if (WINFW_CLEANUP_POLICY_CONTINUE_BLOCKING == cleanupPolicy
		&& FwContext::Policy::Blocked == activePolicy)
	{
		try
		{
			auto engine = wfp::FilterEngine::StandardSession(DEINITIALIZE_TIMEOUT);
			auto sessionController = std::make_unique<SessionController>(std::move(engine));

			rules::persistent::BlockAll blockAll;

			return sessionController->executeTransaction([&](SessionController &controller, wfp::FilterEngine &engine)
			{
				ObjectPurger::GetRemoveNonPersistentFunctor()(engine);

				return controller.addProvider(*MullvadObjects::ProviderPersistent())
					&& controller.addSublayer(*MullvadObjects::SublayerPersistent())
					&& blockAll.apply(controller);
			});
		}
		catch (std::exception & err)
		{
			if (nullptr != g_logSink)
			{
				g_logSink(MULLVAD_LOG_LEVEL_ERROR, err.what(), g_logSinkContext);
			}
			return false;
		}
		catch (...)
		{
			return false;
		}
	}

	return WINFW_POLICY_STATUS_SUCCESS == WinFw_Reset();
}

WINFW_LINKAGE
WINFW_POLICY_STATUS
WINFW_API
WinFw_ApplyPolicyConnecting(
	const WinFwSettings *settings,
	const WinFwRelay *relay,
	const wchar_t *relayClient,
	const PingableHosts *pingableHosts
)
{
	if (nullptr == g_fwContext)
	{
		return WINFW_POLICY_STATUS_GENERAL_FAILURE;
	}

	try
	{
		if (nullptr == settings)
		{
			THROW_ERROR("Invalid argument: settings");
		}

		if (nullptr == relay)
		{
			THROW_ERROR("Invalid argument: relay");
		}

		if (nullptr == relayClient)
		{
			THROW_ERROR("Invalid argument: relayClient");
		}

		return g_fwContext->applyPolicyConnecting(
			*settings,
			*relay,
			relayClient,
			ConvertPingableHosts(pingableHosts)
		) ? WINFW_POLICY_STATUS_SUCCESS : WINFW_POLICY_STATUS_GENERAL_FAILURE;
	}
	catch (common::error::WindowsException &err)
	{
		return HandlePolicyException(err);
	}
	catch (std::exception &err)
	{
		if (nullptr != g_logSink)
		{
			g_logSink(MULLVAD_LOG_LEVEL_ERROR, err.what(), g_logSinkContext);
		}

		return WINFW_POLICY_STATUS_GENERAL_FAILURE;
	}
	catch (...)
	{
		return WINFW_POLICY_STATUS_GENERAL_FAILURE;
	}
}

WINFW_LINKAGE
WINFW_POLICY_STATUS
WINFW_API
WinFw_ApplyPolicyConnected(
	const WinFwSettings *settings,
	const WinFwRelay *relay,
	const wchar_t *relayClient,
	const wchar_t *tunnelInterfaceAlias,
	const wchar_t *v4Gateway,
	const wchar_t *v6Gateway,
	const wchar_t **dnsServers,
	size_t numDnsServers
)
{
	if (nullptr == g_fwContext)
	{
		return WINFW_POLICY_STATUS_GENERAL_FAILURE;
	}

	try
	{
		if (nullptr == settings)
		{
			THROW_ERROR("Invalid argument: settings");
		}

		if (nullptr == relay)
		{
			THROW_ERROR("Invalid argument: relay");
		}

		if (nullptr == relayClient)
		{
			THROW_ERROR("Invalid argument: relayClient");
		}

		if (nullptr == tunnelInterfaceAlias)
		{
			THROW_ERROR("Invalid argument: tunnelInterfaceAlias");
		}

		if (nullptr == v4Gateway)
		{
			THROW_ERROR("Invalid argument: v4Gateway");
		}

		if (nullptr == dnsServers)
		{
			THROW_ERROR("Invalid argument: dnsServers");
		}

		std::vector<wfp::IpAddress> tunnelDnsServers;
		std::vector<wfp::IpAddress> nonTunnelDnsServers;

		const auto v4GatewayIp = wfp::IpAddress(v4Gateway);
		const auto v6GatewayIp = (nullptr != v6Gateway)
			? std::make_optional(wfp::IpAddress(v6Gateway))
			: std::nullopt;

		const auto addToDnsCollection = [&](const std::optional<wfp::IpAddress> &gatewayIp, wfp::IpAddress &&ip)
		{
			if (gatewayIp.has_value() && *gatewayIp == ip)
			{
				// Requests to the gateway IP of the tunnel are only allowed on the tunnel interface.
				tunnelDnsServers.emplace_back(ip);
				return;
			}

			for (const auto &network : g_privateIpRanges)
			{
				if (network.includes(ip))
				{
					//
					// Resolvers on the LAN must be accessible outside the tunnel.
					//

					nonTunnelDnsServers.emplace_back(ip);
					return;
				}
			}

			tunnelDnsServers.emplace_back(ip);
		};

		for (size_t i = 0; i < numDnsServers; i++)
		{
			auto ip = wfp::IpAddress(dnsServers[i]);
			addToDnsCollection(ip.type() == wfp::IpAddress::Type::Ipv4 ? v4GatewayIp : v6GatewayIp, std::move(ip));
		}

		if (nullptr != g_logSink)
		{
			std::stringstream ss;
			ss << "Non-tunnel DNS servers: ";
			for (size_t i = 0; i < nonTunnelDnsServers.size(); i++) {
				if (i > 0)
				{
					ss << ", ";
				}
				ss << common::string::ToAnsi(nonTunnelDnsServers[i].toString());
			}
			g_logSink(MULLVAD_LOG_LEVEL_DEBUG, ss.str().c_str(), g_logSinkContext);

			ss.str(std::string());
			ss << "Tunnel DNS servers: ";
			for (size_t i = 0; i < tunnelDnsServers.size(); i++) {
				if (i > 0)
				{
					ss << ", ";
				}
				ss << common::string::ToAnsi(tunnelDnsServers[i].toString());
			}
			g_logSink(MULLVAD_LOG_LEVEL_DEBUG, ss.str().c_str(), g_logSinkContext);
		}

		return g_fwContext->applyPolicyConnected(
			*settings,
			*relay,
			relayClient,
			tunnelInterfaceAlias,
			tunnelDnsServers,
			nonTunnelDnsServers
		) ? WINFW_POLICY_STATUS_SUCCESS : WINFW_POLICY_STATUS_GENERAL_FAILURE;
	}
	catch (common::error::WindowsException &err)
	{
		return HandlePolicyException(err);
	}
	catch (std::exception &err)
	{
		if (nullptr != g_logSink)
		{
			g_logSink(MULLVAD_LOG_LEVEL_ERROR, err.what(), g_logSinkContext);
		}

		return WINFW_POLICY_STATUS_GENERAL_FAILURE;
	}
	catch (...)
	{
		return WINFW_POLICY_STATUS_GENERAL_FAILURE;
	}
}

WINFW_LINKAGE
WINFW_POLICY_STATUS
WINFW_API
WinFw_ApplyPolicyBlocked(
	const WinFwSettings *settings
)
{
	if (nullptr == g_fwContext)
	{
		return WINFW_POLICY_STATUS_GENERAL_FAILURE;
	}

	try
	{
		if (nullptr == settings)
		{
			THROW_ERROR("Invalid argument: settings");
		}

		return g_fwContext->applyPolicyBlocked(*settings)
			? WINFW_POLICY_STATUS_SUCCESS
			: WINFW_POLICY_STATUS_GENERAL_FAILURE;
	}
	catch (common::error::WindowsException &err)
	{
		return HandlePolicyException(err);
	}
	catch (std::exception &err)
	{
		if (nullptr != g_logSink)
		{
			g_logSink(MULLVAD_LOG_LEVEL_ERROR, err.what(), g_logSinkContext);
		}

		return WINFW_POLICY_STATUS_GENERAL_FAILURE;
	}
	catch (...)
	{
		return WINFW_POLICY_STATUS_GENERAL_FAILURE;
	}
}

WINFW_LINKAGE
WINFW_POLICY_STATUS
WINFW_API
WinFw_Reset()
{
	try
	{
		if (nullptr == g_fwContext)
		{
			return ObjectPurger::Execute(ObjectPurger::GetRemoveAllFunctor())
				? WINFW_POLICY_STATUS_SUCCESS
				: WINFW_POLICY_STATUS_GENERAL_FAILURE;
		}

		return g_fwContext->reset()
			? WINFW_POLICY_STATUS_SUCCESS
			: WINFW_POLICY_STATUS_GENERAL_FAILURE;
	}
	catch (common::error::WindowsException &err)
	{
		return HandlePolicyException(err);
	}
	catch (std::exception &err)
	{
		if (nullptr != g_logSink)
		{
			g_logSink(MULLVAD_LOG_LEVEL_ERROR, err.what(), g_logSinkContext);
		}

		return WINFW_POLICY_STATUS_GENERAL_FAILURE;
	}
	catch (...)
	{
		return WINFW_POLICY_STATUS_GENERAL_FAILURE;
	}
}
