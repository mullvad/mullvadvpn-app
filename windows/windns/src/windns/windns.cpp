#include "stdafx.h"
#include <libcommon/string.h>
#include "windns.h"
#include "confineoperation.h"
#include "netsh.h"
#include "logsink.h"
#include <memory>
#include <vector>
#include <string>
#include <iphlpapi.h>

namespace
{

std::shared_ptr<LogSink> g_LogSink;
std::shared_ptr<NetSh> g_NetSh;

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

uint32_t ConvertInterfaceAliasToIndex(const std::wstring &interfaceAlias)
{
	NET_LUID luid;

	if (NO_ERROR != ConvertInterfaceAliasToLuid(interfaceAlias.c_str(), &luid))
	{
		const auto err = std::wstring(L"Could not resolve LUID of interface: \"")
			.append(interfaceAlias).append(L"\"");

		throw std::runtime_error(common::string::ToAnsi(err).c_str());
	}

	NET_IFINDEX index;

	if (NO_ERROR != ConvertInterfaceLuidToIndex(&luid, &index))
	{
		std::wstringstream ss;

		ss << L"Could not resolve index of interface: \"" << interfaceAlias << L"\""
			<< L"with LUID: 0x" << std::hex << luid.Value;

		throw std::runtime_error(common::string::ToAnsi(ss.str()).c_str());
	}

	return static_cast<uint32_t>(index);
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
	if (g_LogSink)
	{
		return false;
	}

	return ConfineOperation("Initialize", ForwardError, [&]()
	{
		g_LogSink = std::make_shared<LogSink>(LogSinkInfo{ logSink, logContext });

		try
		{
			g_NetSh = std::make_shared<NetSh>(g_LogSink);
		}
		catch (...)
		{
			g_LogSink.reset();
			throw;
		}
	});
}

WINDNS_LINKAGE
bool
WINDNS_API
WinDns_Deinitialize(
)
{
	g_NetSh.reset();
	g_LogSink.reset();

	return true;
}

WINDNS_LINKAGE
bool
WINDNS_API
WinDns_Set(
	const wchar_t *interfaceAlias,
	const wchar_t **ipv4Servers,
	uint32_t numIpv4Servers,
	const wchar_t **ipv6Servers,
	uint32_t numIpv6Servers
)
{
	return ConfineOperation("Apply DNS settings", ForwardError, [&]()
	{
		const auto interfaceIndex = ConvertInterfaceAliasToIndex(interfaceAlias);

		if (nullptr != ipv4Servers && 0 != numIpv4Servers)
		{
			g_NetSh->setIpv4StaticDns(interfaceIndex, MakeStringArray(ipv4Servers, numIpv4Servers));
		}

		if (nullptr != ipv6Servers && 0 != numIpv6Servers)
		{
			g_NetSh->setIpv6StaticDns(interfaceIndex, MakeStringArray(ipv6Servers, numIpv6Servers));
		}
	});
}
