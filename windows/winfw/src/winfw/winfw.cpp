#include "stdafx.h"
#include "winfw.h"
#include "fwcontext.h"
#include "libwfp/ipaddress.h"
#include <windows.h>
#include <stdexcept>

namespace
{

uint32_t g_timeout = 0;

WinFwErrorSink g_ErrorSink = nullptr;
void * g_ErrorContext = nullptr;

FwContext *g_fwContext = nullptr;

} // anonymous namespace

WINFW_LINKAGE
bool
WINFW_API
WinFw_Initialize(
	uint32_t timeout,
	WinFwErrorSink errorSink,
	void *errorContext
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

	g_ErrorSink = errorSink;
	g_ErrorContext = errorContext;

	try
	{
		g_fwContext = new FwContext(g_timeout);
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
	const WinFwRelay &relay
)
{
	if (nullptr == g_fwContext)
	{
		return false;
	}

	try
	{
		return g_fwContext->applyPolicyConnecting(settings, relay);
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

WINFW_LINKAGE
bool
WINFW_API
WinFw_ApplyPolicyConnected(
	const WinFwSettings &settings,
	const WinFwRelay &relay,
	const wchar_t *tunnelInterfaceAlias,
	const wchar_t *v4Gateway,
	const wchar_t *v6Gateway
)
{
	if (nullptr == g_fwContext)
	{
		return false;
	}

	try
	{
		return g_fwContext->applyPolicyConnected(settings, relay, tunnelInterfaceAlias, v4Gateway, v6Gateway);
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

WINFW_LINKAGE
bool
WINFW_API
WinFw_Reset()
{
	if (nullptr == g_fwContext)
	{
		//
		// This is OK because the practical difference between having no instance
		// and having a reset instance is negligible.
		//
		return true;
	}

	try
	{
		return g_fwContext->reset();
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
