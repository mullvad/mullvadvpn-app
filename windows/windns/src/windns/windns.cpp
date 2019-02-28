#include "stdafx.h"
#include "windns.h"
#include "windnscontext.h"
#include "clientsinkinfo.h"
#include "confineoperation.h"
#include "recoveryformatter.h"
#include "recoverylogic.h"
#include "netsh.h"
#include "logsink.h"
#include <memory>
#include <vector>
#include <string>

namespace
{

LogSink *g_LogSink = nullptr;
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

void ForwardError(const char *message, const char **details, uint32_t numDetails)
{
	if (nullptr != g_LogSink)
	{
		g_LogSink->error(message, details, numDetails);
	}
}

} // anonymous namespace

WINDNS_LINKAGE
bool
WINDNS_API
WinDns_Initialize(
	WinDnsLogSink logSink,
	void *logContext
)
{
	if (nullptr != g_Context)
	{
		return false;
	}

	return ConfineOperation("Initialize", ForwardError, [&]()
	{
		if (nullptr == g_LogSink)
		{
			g_LogSink = new LogSink(LogSinkInfo{ logSink, logContext });
			NetSh::Construct(g_LogSink);
		}
		else
		{
			g_LogSink->setTarget(LogSinkInfo{ logSink, logContext });
		}

		g_Context = new WinDnsContext(g_LogSink);
	});
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

	// Maintain a single instance forever and invoke setTarget() on it.
	//delete g_LogSink;
	//g_LogSink = nullptr;

	return true;
}

WINDNS_LINKAGE
bool
WINDNS_API
WinDns_Set(
	const wchar_t **ipv4Servers,
	uint32_t numIpv4Servers,
	const wchar_t **ipv6Servers,
	uint32_t numIpv6Servers,
	WinDnsRecoverySink recoverySink,
	void *recoveryContext
)
{
	if (nullptr == g_Context
		|| nullptr == ipv4Servers
		|| 0 == numIpv4Servers
		|| nullptr == recoverySink)
	{
		return false;
	}

	return ConfineOperation("Enforce DNS settings", ForwardError, [&]()
	{
		g_Context->set(MakeStringArray(ipv4Servers, numIpv4Servers), MakeStringArray(ipv6Servers, \
			numIpv6Servers), RecoverySinkInfo{ recoverySink, recoveryContext });
	});
}

WINDNS_LINKAGE
bool
WINDNS_API
WinDns_Reset(
)
{
	if (nullptr == g_Context)
	{
		return true;
	}

	return ConfineOperation("Reset DNS settings", ForwardError, []()
	{
		g_Context->reset();
	});
}

WINDNS_LINKAGE
bool
WINDNS_API
WinDns_Recover(
	const void *recoveryData,
	uint32_t dataLength
)
{
	return ConfineOperation("Recover DNS settings", ForwardError, [&]()
	{
		auto unpacked = RecoveryFormatter::Unpack(reinterpret_cast<const uint8_t *>(recoveryData), dataLength);

		static const uint32_t TIMEOUT_TEN_SECONDS = 1000 * 10;

		RecoveryLogic::RestoreInterfaces(unpacked, g_LogSink, TIMEOUT_TEN_SECONDS);
	});
}
