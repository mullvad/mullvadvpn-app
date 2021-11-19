#include "stdafx.h"
#include <libcommon/guid.h>
#include <libcommon/string.h>
#include <libcommon/error.h>
#include <libshared/network/interfaceutils.h>
#include <libcommon/logging/ilogsink.h>
#include <libshared/logging/logsinkadapter.h>
#include "windns.h"
#include "confineoperation.h"
#include "netsh.h"
#include <memory>
#include <vector>
#include <string>
#include <sstream>
#include <ws2tcpip.h>
#include <ws2ipdef.h>
#include <winsock2.h>	// magic order :-(
#include <iphlpapi.h>	// if we don't do this then most of iphlpapi is not actually defined

bool operator==(const IN_ADDR &lhs, const IN_ADDR &rhs)
{
	return 0 == memcmp(&lhs, &rhs, sizeof(IN_ADDR));
}

bool operator==(const IN6_ADDR &lhs, const IN6_ADDR &rhs)
{
	return 0 == memcmp(&lhs, &rhs, sizeof(IN6_ADDR));
}

namespace
{

std::shared_ptr<common::logging::ILogSink> g_LogSink;
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

uint32_t ConvertInterfaceAliasToIndex(const std::wstring &interfaceAlias)
{
	NET_LUID luid;

	if (NO_ERROR != ConvertInterfaceAliasToLuid(interfaceAlias.c_str(), &luid))
	{
		const auto err = std::wstring(L"Could not resolve LUID of interface: \"")
			.append(interfaceAlias).append(L"\"");

		THROW_ERROR(common::string::ToAnsi(err).c_str());
	}

	NET_IFINDEX index;

	if (NO_ERROR != ConvertInterfaceLuidToIndex(&luid, &index))
	{
		std::wstringstream ss;

		ss << L"Could not resolve index of interface: \"" << interfaceAlias << L"\""
			<< L"with LUID: 0x" << std::hex << luid.Value;

		THROW_ERROR(common::string::ToAnsi(ss.str()).c_str());
	}

	return static_cast<uint32_t>(index);
}

struct AdapterDnsAddresses
{
	std::vector<IN_ADDR> ipv4;
	std::vector<IN6_ADDR> ipv6;
};

//
// Use name when finding the adapter to be more resilient over time.
// The adapter structure that is returned has two fields for interface index.
// If IPv4 is enabled, 'IfIndex' will be set. Otherwise set to 0.
// If IPv6 is enabled, 'Ipv6IfIndex' will be set. Otherwise set to 0.
// If both IPv4 and IPv6 is enabled, then both fields will be set, and have the same value.
//
AdapterDnsAddresses GetAdapterDnsAddresses(const std::wstring &adapterAlias)
{
	using shared::network::InterfaceUtils;

	const auto adapters = InterfaceUtils::GetAllAdapters(
		AF_UNSPEC,
		GAA_FLAG_SKIP_UNICAST | GAA_FLAG_SKIP_ANYCAST | GAA_FLAG_SKIP_MULTICAST
	);

	NET_LUID luid;
	if (NO_ERROR != ConvertInterfaceAliasToLuid(adapterAlias.c_str(), &luid))
	{
		const auto err = std::wstring(L"Could not resolve LUID of interface: \"")
			.append(adapterAlias).append(L"\"");
		THROW_ERROR(common::string::ToAnsi(err).c_str());
	}

	for (const auto adapter : adapters)
	{
		if (luid.Value != adapter.raw().Luid.Value)
		{
			continue;
		}

		AdapterDnsAddresses out;

		for (auto server = adapter.raw().FirstDnsServerAddress; nullptr != server; server = server->Next)
		{
			if (AF_INET == server->Address.lpSockaddr->sa_family)
			{
				out.ipv4.push_back(((const SOCKADDR_IN*)server->Address.lpSockaddr)->sin_addr);
			}
			else if (AF_INET6 == server->Address.lpSockaddr->sa_family)
			{
				out.ipv6.push_back(((const SOCKADDR_IN6_LH*)server->Address.lpSockaddr)->sin6_addr);
			}
		}

		return out;
	}

	const auto msg = std::string("Could not find interface with alias: ")
		.append(common::string::ToAnsi(adapterAlias));

	THROW_ERROR(msg.c_str());
}

AdapterDnsAddresses ConvertAddresses(
	const wchar_t **ipv4Servers,
	uint32_t numIpv4Servers,
	const wchar_t **ipv6Servers,
	uint32_t numIpv6Servers
)
{
	AdapterDnsAddresses out;

	if (nullptr != ipv4Servers && 0 != numIpv4Servers)
	{
		for (uint32_t i = 0; i < numIpv4Servers; ++i)
		{
			IN_ADDR converted;

			if (1 != InetPtonW(AF_INET, ipv4Servers[i], &converted))
			{
				THROW_ERROR("Failed to convert IPv4 address");
			}

			out.ipv4.push_back(converted);
		}
	}

	if (nullptr != ipv6Servers && 0 != numIpv6Servers)
	{
		for (uint32_t i = 0; i < numIpv6Servers; ++i)
		{
			IN6_ADDR converted;

			if (1 != InetPtonW(AF_INET6, ipv6Servers[i], &converted))
			{
				THROW_ERROR("Failed to convert IPv6 address");
			}

			out.ipv6.push_back(converted);
		}
	}

	return out;
}

bool Equal(const AdapterDnsAddresses &lhs, const AdapterDnsAddresses &rhs)
{
	return lhs.ipv4 == rhs.ipv4
		&& lhs.ipv6 == rhs.ipv6;
}

} // anonymous namespace

WINDNS_LINKAGE
bool
WINDNS_API
WinDns_Initialize(
	MullvadLogSink logSink,
	void *logSinkContext
)
{
	if (g_LogSink)
	{
		return false;
	}

	try
	{
		g_LogSink = std::make_shared<shared::logging::LogSinkAdapter>(logSink, logSinkContext);

		try
		{
			g_NetSh = std::make_shared<NetSh>(g_LogSink);
		}
		catch (...)
		{
			g_LogSink.reset();
			throw;
		}

		return true;
	}
	catch (const std::exception &err)
	{
		if (nullptr != logSink)
		{
			const auto msg = std::string("Failed to initialize WinDns: ").append(err.what());
			logSink(MULLVAD_LOG_LEVEL_ERROR, msg.c_str(), logSinkContext);
		}

		return false;
	}
	catch (...)
	{
		if (nullptr != logSink)
		{
			const std::string msg("Failed to initialize WinDns: Unspecified error");
			logSink(MULLVAD_LOG_LEVEL_ERROR, msg.c_str(), logSinkContext);
		}

		return false;
	}
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
	if (nullptr == g_LogSink)
	{
		return false;
	}

	if (nullptr == interfaceAlias)
	{
		g_LogSink->error("Invalid argument: interfaceAlias");
		return false;
	}

	//
	// Check the settings on the adapter.
	// If it already has the exact same settings we need, we're done.
	//

	try
	{
		const auto activeSettings = GetAdapterDnsAddresses(interfaceAlias);
		const auto wantedSetting = ConvertAddresses(ipv4Servers, numIpv4Servers, ipv6Servers, numIpv6Servers);

		if (Equal(activeSettings, wantedSetting))
		{
			std::stringstream ss;

			ss << "DNS settings on adapter with alias \"" << common::string::ToAnsi(interfaceAlias)
				<< "\" are up-to-date";

			g_LogSink->info(ss.str().c_str());

			return true;
		}
	}
	catch (const std::exception &err)
	{
		std::stringstream ss;

		ss << "Failed to evaluate DNS settings on adapter with alias \""
			<< common::string::ToAnsi(interfaceAlias) << "\": " << err.what();

		g_LogSink->info(ss.str().c_str());
	}
	catch (...)
	{
		std::stringstream ss;

		ss << "Failed to evaluate DNS settings on adapter with alias \""
			<< common::string::ToAnsi(interfaceAlias) << "\": Unspecified failure";

		g_LogSink->info(ss.str().c_str());
	}

	//
	// Apply specified settings.
	//

	const auto operation = std::string("Apply DNS settings on adapter with alias \"")
		.append(common::string::ToAnsi(interfaceAlias)).append("\"");

	return ConfineOperation(operation.c_str(), g_LogSink, [&]()
	{
		const auto interfaceIndex = ConvertInterfaceAliasToIndex(interfaceAlias);

		if (nullptr != ipv4Servers && 0 != numIpv4Servers)
		{
			g_NetSh->setIpv4StaticDns(interfaceIndex, MakeStringArray(ipv4Servers, numIpv4Servers));
		}
		else
		{
			// This is required to clear any current settings.
			g_NetSh->setIpv4DhcpDns(interfaceIndex);
		}

		if (nullptr != ipv6Servers && 0 != numIpv6Servers)
		{
			g_NetSh->setIpv6StaticDns(interfaceIndex, MakeStringArray(ipv6Servers, numIpv6Servers));
		}
		else
		{
			// This is required to clear any current settings.
			g_NetSh->setIpv6DhcpDns(interfaceIndex);
		}
	});
}
