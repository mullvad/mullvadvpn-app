#include "stdafx.h"
#include "winfw.h"
#include "fwcontext.h"
#include "objectpurger.h"
#include "mullvadobjects.h"
#include "rules/persistent/blockall.h"
#include <windows.h>
#include <libcommon/error.h>
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
	const wchar_t *v4DnsHost,
	const wchar_t *v6DnsHost
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

		if (nullptr == v4DnsHost)
		{
			THROW_ERROR("Invalid argument: v4DnsHost");
		}

		std::vector<wfp::IpAddress> tunnelDnsServers = { wfp::IpAddress(v4DnsHost) };

		if (nullptr != v6DnsHost)
		{
			tunnelDnsServers.emplace_back(wfp::IpAddress(v6DnsHost));
		}

		return g_fwContext->applyPolicyConnected(
			*settings,
			*relay,
			relayClient,
			tunnelInterfaceAlias,
			tunnelDnsServers
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
