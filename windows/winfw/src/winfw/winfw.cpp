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
	const WinFwEndpoint *relay,
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

		if (nullptr == relay)
		{
			THROW_ERROR("Invalid argument: relay");
		}

		if (nullptr == allowedTunnelTraffic)
		{
			THROW_ERROR("Invalid argument: allowedTunnelTraffic");
		}

		std::vector<std::wstring> relayClientWstrings;
		relayClientWstrings.reserve(relayClientsLen);
		for(int i = 0; i < relayClientsLen; i++) {
			relayClientWstrings.push_back(relayClients[i]);
		}

		return g_fwContext->applyPolicyConnecting(
			*settings,
			*relay,
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
	const WinFwEndpoint *relay,
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

		if (nullptr == relay)
		{
			THROW_ERROR("Invalid argument: relay");
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
		for(int i = 0; i < relayClientsLen; i++) {
			relayClientWstrings.push_back(relayClients[i]);
		}

		return g_fwContext->applyPolicyConnected(
			*settings,
			*relay,
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
