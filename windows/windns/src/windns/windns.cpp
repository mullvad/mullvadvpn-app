#include "stdafx.h"
#include "windns.h"
#include "windnscontext.h"
#include "clientsinkinfo.h"
#include <vector>
#include <string>

namespace
{

WinDnsErrorSink g_ErrorSink = nullptr;
void *g_ErrorContext = nullptr;

WinDnsContext *g_Context = nullptr;

std::vector<std::wstring> MakeStringArray(const wchar_t **strings, uint32_t numStrings)
{
	std::vector<std::wstring> v;

	while (numStrings--)
	{
		v.emplace_back(*strings++);
	}

	return v;
}

} // anonymous namespace

WINDNS_LINKAGE
bool
WINDNS_API
WinDns_Initialize(
	WinDnsErrorSink errorSink,
	void *errorContext
)
{
	if (nullptr != g_Context)
	{
		return false;
	}

	g_ErrorSink = errorSink;
	g_ErrorContext = errorContext;

	try
	{
		g_Context = new WinDnsContext;
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

WINDNS_LINKAGE
bool
WINDNS_API
WinDns_Deinitialize(
)
{
	if (nullptr == g_Context)
	{
		return true;
	}

	delete g_Context;
	g_Context = nullptr;

	return true;
}

WINDNS_LINKAGE
bool
WINDNS_API
WinDns_Set(
	const wchar_t **servers,
	uint32_t numServers,
	WinDnsConfigSink configSink,
	void *configContext
)
{
	if (nullptr == g_Context
		|| 0 == numServers
		|| nullptr == configSink)
	{
		return false;
	}

	try
	{
		//
		// TODO: This is a temporary hack to enable alpha version. Review.
		//

		delete g_Context;
		g_Context = nullptr;

		g_Context = new WinDnsContext;

		//
		// Onwards.
		//

		ClientSinkInfo sinkInfo;

		sinkInfo.errorSinkInfo = ErrorSinkInfo{ g_ErrorSink, g_ErrorContext };
		sinkInfo.configSinkInfo = ConfigSinkInfo{ configSink, configContext };

		return g_Context->set(MakeStringArray(servers, numServers), sinkInfo);
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

WINDNS_LINKAGE
bool
WINDNS_API
WinDns_Reset(
)
{
	if (nullptr == g_Context)
	{
		return false;
	}

	try
	{
		return g_Context->reset();
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

WINDNS_LINKAGE
bool
WINDNS_API
WinDns_Recover(
	const void *configData,
	uint32_t dataLength
)
{
	// TODO: smart stuff

	return false;
}
