#include "stdafx.h"
#include "wfpctl.h"
#include "wfpcontext.h"
#include "libwfp/ipaddress.h"
#include <windows.h>
#include <stdexcept>

namespace
{

uint32_t g_timeout = 0;

WfpctlErrorSink g_ErrorSink = nullptr;
void * g_ErrorContext = nullptr;

WfpContext *g_wfpContext = nullptr;

} // anonymous namespace

WFPCTL_LINKAGE
bool
WFPCTL_API
Wfpctl_Initialize(
	uint32_t timeout,
	WfpctlErrorSink errorSink,
	void *errorContext
)
{
	if (nullptr != g_wfpContext)
	{
		//
		// This is an error.
		// The existing instance may have a different timeout etc.
		//
		return false;
	}

	// Convert seconds to milliseconds.
	g_timeout = timeout * 1000;

	g_ErrorSink = errorSink;
	g_ErrorContext = errorContext;

	try
	{
		g_wfpContext = new WfpContext(g_timeout);
	}
	catch (std::exception &err)
	{
		if (nullptr != g_ErrorSink)
		{
			g_ErrorSink(err.what(), g_ErrorContext);
		}

		return false;
	}
	catch (...)
	{
		return false;
	}

	return true;
}

WFPCTL_LINKAGE
bool
WFPCTL_API
Wfpctl_Deinitialize()
{
	if (nullptr == g_wfpContext)
	{
		return true;
	}

	delete g_wfpContext;
	g_wfpContext = nullptr;

	return true;
}

WFPCTL_LINKAGE
bool
WFPCTL_API
Wfpctl_ApplyPolicyConnecting(
	const WfpctlSettings &settings,
	const WfpctlRelay &relay
)
{
	if (nullptr == g_wfpContext)
	{
		return false;
	}

	try
	{
		return g_wfpContext->applyPolicyConnecting(settings, relay);
	}
	catch (std::exception &err)
	{
		if (nullptr != g_ErrorSink)
		{
			g_ErrorSink(err.what(), g_ErrorContext);
		}

		return false;
	}
	catch (...)
	{
		return false;
	}
}

WFPCTL_LINKAGE
bool
WFPCTL_API
Wfpctl_ApplyPolicyConnected(
	const WfpctlSettings &settings,
	const WfpctlRelay &relay,
	const wchar_t *tunnelInterfaceAlias,
	const wchar_t *primaryDns
)
{
	if (nullptr == g_wfpContext)
	{
		return false;
	}

	try
	{
		return g_wfpContext->applyPolicyConnected(settings, relay, tunnelInterfaceAlias, primaryDns);
	}
	catch (std::exception &err)
	{
		if (nullptr != g_ErrorSink)
		{
			g_ErrorSink(err.what(), g_ErrorContext);
		}

		return false;
	}
	catch (...)
	{
		return false;
	}
}

WFPCTL_LINKAGE
bool
WFPCTL_API
Wfpctl_Reset()
{
	if (nullptr == g_wfpContext)
	{
		//
		// This is an error.
		// The reset instance enforces the block-all rule.
		//
		return false;
	}

	try
	{
		return g_wfpContext->reset();
	}
	catch (std::exception &err)
	{
		if (nullptr != g_ErrorSink)
		{
			g_ErrorSink(err.what(), g_ErrorContext);
		}

		return false;
	}
	catch (...)
	{
		return false;
	}
}
