#include "stdafx.h"
#include "winfw.h"
#include "fwcontext.h"
#include "objectpurger.h"
#include "mullvadobjects.h"
#include "rules/persistent/blockall.h"
#include "rules/baseline/blockall.h"
#include "libwfp/ipnetwork.h"
#include "libwfp/filterengine.h"
#include "libwfp/objectenumerator.h"
#include <windows.h>
#include <libcommon/error.h>
#include <libcommon/string.h>
#include <optional>
#include <psapi.h>
#include <sstream>
#include <set>

namespace
{

constexpr uint32_t DEINITIALIZE_TIMEOUT = 5000;

MullvadLogSink g_logSink = nullptr;
void *g_logSinkContext = nullptr;

FwContext *g_fwContext = nullptr;

// Log the filename of active WFP sessions as comma-separated values. Note that the path is not logged.
// If the process can not be opened or its filename can not be obtained, the process ID is logged instead.
void
LogActiveWfpSessions()
{
	if (nullptr == g_logSink)
	{
		return;
	}

	try
	{
		auto engine = wfp::FilterEngine::DynamicSession();
		std::set<DWORD> sessionPids;

		wfp::ObjectEnumerator::Sessions(*engine, [&sessionPids](const FWPM_SESSION0 &session) -> bool
		{
			if (session.processId != 0)
			{
				sessionPids.insert(session.processId);
			}
			return true;
		});

		if (sessionPids.empty())
		{
			g_logSink(MULLVAD_LOG_LEVEL_DEBUG, "No active WFP sessions found", g_logSinkContext);
			return;
		}

		std::stringstream ss;
		ss << "Active WFP sessions from processes: ";
		bool first = true;

		for (DWORD pid : sessionPids)
		{
			if (!first)
			{
				ss << ", ";
			}
			first = false;

			HANDLE hProcess = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, FALSE, pid);
			if (nullptr == hProcess)
			{
				// Log pid only if we cannot open the process
				ss << "PID:" << pid;
				continue;
			}

			wchar_t processPath[MAX_PATH];
			DWORD pathSize = MAX_PATH;
			if (QueryFullProcessImageNameW(hProcess, 0, processPath, &pathSize))
			{
				// Extract just the filename
				wchar_t *filename = wcsrchr(processPath, L'\\');
				if (nullptr != filename)
				{
					filename++;
				}
				else
				{
					filename = processPath;
				}
				ss << common::string::ToAnsi(filename);
			}
			else
			{
				// Log pid only if we cannot obtain the path
				ss << "PID:" << pid;
			}
			CloseHandle(hProcess);
		}

		g_logSink(MULLVAD_LOG_LEVEL_DEBUG, ss.str().c_str(), g_logSinkContext);
	}
	catch (...)
	{
		g_logSink(MULLVAD_LOG_LEVEL_ERROR, "Failed to log WFP sessions", g_logSinkContext);
	}
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
		// Log processes potentially holding the transaction lock
		LogActiveWfpSessions();

		return WINFW_POLICY_STATUS_LOCK_TIMEOUT;
	}

	return WINFW_POLICY_STATUS_GENERAL_FAILURE;
}

template<typename T>
std::optional<T> MakeOptional(T* object)
{
	if (nullptr == object)
	{
		return std::nullopt;
	}
	return std::make_optional(*object);
}

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
	const WinFwAllowedEndpoint *allowedEndpoint,
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

		g_fwContext = new FwContext(timeout_ms, *settings, MakeOptional(allowedEndpoint));
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

	if (nullptr != g_logSink)
	{
		g_logSink(MULLVAD_LOG_LEVEL_DEBUG, "Deinitializing WinFw", g_logSinkContext);
	}

	//
	// Continue blocking with persistent rules if this is what the caller requested
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

			if (nullptr != g_logSink)
			{
				g_logSink(MULLVAD_LOG_LEVEL_DEBUG, "Adding persistent block rules", g_logSinkContext);
			}

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

	//
	// Continue blocking with non-persistent rules if this is what the caller requested
	// and if the current policy is "(net) blocked".
	//
	if (WINFW_CLEANUP_POLICY_BLOCK_UNTIL_REBOOT == cleanupPolicy
		&& FwContext::Policy::Blocked == activePolicy)
	{
		if (nullptr != g_logSink)
		{
			g_logSink(MULLVAD_LOG_LEVEL_DEBUG, "Keeping ephemeral block rules", g_logSinkContext);
		}

		// All we have to is *not* call WinFw_Reset, since blocking filters have been applied.
		return true;
	}

	return WINFW_POLICY_STATUS_SUCCESS == WinFw_Reset();
}

WINFW_LINKAGE
WINFW_POLICY_STATUS
WINFW_API
WinFw_ApplyPolicyConnecting(
	const WinFwSettings *settings,
	size_t numRelays,
	const WinFwEndpoint *relays,
	const wchar_t *exitEndpointIp,
	const wchar_t **relayClients,
	size_t relayClientsLen,
	const wchar_t *tunnelInterfaceAlias,
	const WinFwAllowedEndpoint *allowedEndpoint,
	const WinFwAllowedTunnelTraffic *allowedTunnelTraffic
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

		if (nullptr == relays)
		{
			THROW_ERROR("Invalid argument: relays");
		}

		if (0 == numRelays)
		{
			THROW_ERROR("Invalid argument: numRelays");
		}

		if (nullptr == allowedTunnelTraffic)
		{
			THROW_ERROR("Invalid argument: allowedTunnelTraffic");
		}

		std::vector<WinFwEndpoint> relayEndpoints;
		relayEndpoints.reserve(numRelays);
		for (size_t i = 0; i < numRelays; i++)
		{
			relayEndpoints.push_back(relays[i]);
		}

		const auto exitIpAddr = (exitEndpointIp != nullptr) ? std::make_optional(wfp::IpAddress(exitEndpointIp)) : std::nullopt;

		for (const auto &entryEndpoint : relayEndpoints)
		{
			const auto ipAddr = wfp::IpAddress(entryEndpoint.ip);
			if (ipAddr == exitIpAddr)
			{
				THROW_ERROR("Invalid argument: relay IP must not equal exitEndpointIp");
			}
		}

		std::vector<std::wstring> relayClientWstrings;
		relayClientWstrings.reserve(relayClientsLen);
		for (size_t i = 0; i < relayClientsLen; i++) {
			relayClientWstrings.push_back(relayClients[i]);
		}

		return g_fwContext->applyPolicyConnecting(
			*settings,
			relayEndpoints,
			exitIpAddr,
			relayClientWstrings,
			tunnelInterfaceAlias != nullptr ? std::make_optional(tunnelInterfaceAlias) : std::nullopt,
			MakeOptional(allowedEndpoint),
			*allowedTunnelTraffic
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
	size_t numRelays,
	const WinFwEndpoint *relays,
	const wchar_t *exitEndpointIp,
	const wchar_t **relayClients,
	size_t relayClientsLen,
	const wchar_t *tunnelInterfaceAlias,
	const wchar_t * const *tunnelDnsServers,
	size_t numTunnelDnsServers,
	const wchar_t * const *nonTunnelDnsServers,
	size_t numNonTunnelDnsServers
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

		if (nullptr == relays)
		{
			THROW_ERROR("Invalid argument: relays");
		}

		if (0 == numRelays)
		{
			THROW_ERROR("Invalid argument: numRelays");
		}

		if (nullptr == tunnelInterfaceAlias)
		{
			THROW_ERROR("Invalid argument: tunnelInterfaceAlias");
		}

		if (nullptr == tunnelDnsServers)
		{
			THROW_ERROR("Invalid argument: tunnelDnsServers");
		}

		if (nullptr == nonTunnelDnsServers)
		{
			THROW_ERROR("Invalid argument: nonTunnelDnsServers");
		}

		std::vector<WinFwEndpoint> relayEndpoints;
		relayEndpoints.reserve(numRelays);
		for (size_t i = 0; i < numRelays; i++)
		{
			relayEndpoints.push_back(relays[i]);
		}

		const auto exitIpAddr = (exitEndpointIp != nullptr) ? std::make_optional(wfp::IpAddress(exitEndpointIp)) : std::nullopt;

		for (const auto &entryEndpoint : relayEndpoints)
		{
			const auto ipAddr = wfp::IpAddress(entryEndpoint.ip);
			if (ipAddr == exitIpAddr)
			{
				THROW_ERROR("Invalid argument: relay IP must not equal exitEndpointIp");
			}
		}

		std::vector<wfp::IpAddress> convertedTunnelDnsServers;
		std::vector<wfp::IpAddress> convertedNonTunnelDnsServers;

		for (size_t i = 0; i < numTunnelDnsServers; i++)
		{
			auto ip = wfp::IpAddress(tunnelDnsServers[i]);
			convertedTunnelDnsServers.push_back(ip);
		}
		for (size_t i = 0; i < numNonTunnelDnsServers; i++)
		{
			auto ip = wfp::IpAddress(nonTunnelDnsServers[i]);
			convertedNonTunnelDnsServers.push_back(ip);
		}

		if (nullptr != g_logSink)
		{
			std::stringstream ss;
			ss << "Non-tunnel DNS servers: ";
			for (size_t i = 0; i < convertedNonTunnelDnsServers.size(); i++) {
				if (i > 0)
				{
					ss << ", ";
				}
				ss << common::string::ToAnsi(convertedNonTunnelDnsServers[i].toString());
			}
			g_logSink(MULLVAD_LOG_LEVEL_DEBUG, ss.str().c_str(), g_logSinkContext);

			ss.str(std::string());
			ss << "Tunnel DNS servers: ";
			for (size_t i = 0; i < convertedTunnelDnsServers.size(); i++) {
				if (i > 0)
				{
					ss << ", ";
				}
				ss << common::string::ToAnsi(convertedTunnelDnsServers[i].toString());
			}
			g_logSink(MULLVAD_LOG_LEVEL_DEBUG, ss.str().c_str(), g_logSinkContext);
		}

		std::vector<std::wstring> relayClientWstrings;
		relayClientWstrings.reserve(relayClientsLen);
		for (size_t i = 0; i < relayClientsLen; i++) {
			relayClientWstrings.push_back(relayClients[i]);
		}

		return g_fwContext->applyPolicyConnected(
			*settings,
			relayEndpoints,
			exitIpAddr,
			relayClientWstrings,
			tunnelInterfaceAlias,
			convertedTunnelDnsServers,
			convertedNonTunnelDnsServers
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
	const WinFwSettings *settings,
	const WinFwAllowedEndpoint *allowedEndpoint
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

		return g_fwContext->applyPolicyBlocked(*settings, MakeOptional(allowedEndpoint))
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
