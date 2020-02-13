#include "stdafx.h"
#include "winfw.h"
#include "fwcontext.h"
#include "objectpurger.h"
#include <windows.h>
#include <libcommon/error.h>
#include <optional>

namespace
{

uint32_t g_timeout = 0;

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
	if (nullptr != g_fwContext)
	{
		//
		// This is an error.
		// The existing instance may have a different timeout etc.
		//
		return false;
	}

	// Convert seconds to milliseconds.
	g_timeout = timeout * 1000;

	g_logSink = logSink;
	g_logSinkContext = logSinkContext;

	try
	{
		g_fwContext = new FwContext(g_timeout);
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
	const WinFwSettings &settings,
	MullvadLogSink logSink,
	void *logSinkContext
)
{
	if (nullptr != g_fwContext)
	{
		//
		// This is an error.
		// The existing instance may have a different timeout etc.
		//
		return false;
	}

	// Convert seconds to milliseconds.
	g_timeout = timeout * 1000;

	g_logSink = logSink;
	g_logSinkContext = logSinkContext;

	try
	{
		g_fwContext = new FwContext(g_timeout, settings);
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
WinFw_Deinitialize()
{
	if (nullptr == g_fwContext)
	{
		return true;
	}

	delete g_fwContext;
	g_fwContext = nullptr;

	return true;
}

WINFW_LINKAGE
bool
WINFW_API
WinFw_ApplyPolicyConnecting(
	const WinFwSettings &settings,
	const WinFwRelay &relay,
	const PingableHosts *pingableHosts
)
{
	if (nullptr == g_fwContext)
	{
		return false;
	}

	try
	{
		return g_fwContext->applyPolicyConnecting(settings, relay, ConvertPingableHosts(pingableHosts));
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
}

WINFW_LINKAGE
bool
WINFW_API
WinFw_ApplyPolicyConnected(
	const WinFwSettings &settings,
	const WinFwRelay &relay,
	const wchar_t *tunnelInterfaceAlias,
	const wchar_t *v4DnsHost,
	const wchar_t *v6DnsHost
)
{
	if (nullptr == g_fwContext)
	{
		return false;
	}

	try
	{
		return g_fwContext->applyPolicyConnected(
			settings,
			relay,
			tunnelInterfaceAlias,
			wfp::IpAddress(v4DnsHost),
			nullptr != v6DnsHost ? std::make_optional(wfp::IpAddress(v6DnsHost)) : std::nullopt
		);
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
}

WINFW_LINKAGE
bool
WINFW_API
WinFw_ApplyPolicyBlocked(
	const WinFwSettings &settings
)
{
	if (nullptr == g_fwContext)
	{
		return false;
	}

	try
	{
		return g_fwContext->applyPolicyBlocked(settings);
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
}

WINFW_LINKAGE
bool
WINFW_API
WinFw_Reset()
{
	try
	{
		if (nullptr == g_fwContext)
		{
			return ObjectPurger::Execute(ObjectPurger::GetRemoveAllFunctor());
		}

		return g_fwContext->reset();
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
}
