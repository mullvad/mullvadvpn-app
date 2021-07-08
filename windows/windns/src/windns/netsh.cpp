#include "stdafx.h"
#include "netsh.h"
#include <libcommon/string.h>
#include <libcommon/error.h>
#include <libcommon/filesystem.h>
#include <libcommon/guid.h>
#include <sstream>
#include <stdexcept>
#include <filesystem>
#include <iphlpapi.h>

namespace
{

__declspec(noreturn) void ThrowWithDetails(std::string &&error, common::process::ApplicationRunner &netsh)
{
	std::string details("Failed to capture output from 'netsh'");

	std::string output;

	static const size_t MAX_CHARS = 2048;
	static const size_t TIMEOUT_MILLISECONDS = 2000;

	if (netsh.read(output, MAX_CHARS, TIMEOUT_MILLISECONDS))
	{
		details = std::move(output);
	}

	const auto msg = std::string(error).append(": ").append(details);

	THROW_ERROR(msg.c_str());
}

} // anonymous namespace

NetSh::NetSh(std::shared_ptr<common::logging::ILogSink> logSink)
	: m_logSink(logSink)
{
	const auto system32 = common::fs::GetKnownFolderPath(FOLDERID_System);
	m_netShPath = std::filesystem::path(system32).append(L"netsh.exe");
}

void NetSh::setIpv4StaticDns(uint32_t interfaceIndex,
	const std::vector<std::wstring> &nameServers, uint32_t timeout)
{
	//
	// Setting primary and secondary name server requires two invokations:
	//
	// netsh interface ipv4 set dnsservers name="Ethernet 2" source=static address=8.8.8.8 validate=no
	// netsh interface ipv4 add dnsservers name="Ethernet 2" address=8.8.4.4 index=2 validate=no
	//
	// Note: we're specifying the interface by index instead.
	//

	if (nameServers.empty())
	{
		THROW_ERROR("Invalid list of name servers (zero length list)");
	}

	{
		std::wstringstream ss;

		ss << L"interface ipv4 set dnsservers name="
			<< interfaceIndex
			<< L" source=static address="
			<< nameServers[0]
			<< L" validate=no";

		auto netsh = common::process::ApplicationRunner::StartWithoutConsole(m_netShPath, ss.str());

		validateShellOut(*netsh, timeout);
	}

	//
	// Set additional name servers.
	//

	for (size_t i = 1; i < nameServers.size(); ++i)
	{
		std::wstringstream ss;

		ss << L"interface ipv4 add dnsservers name="
			<< interfaceIndex
			<< L" address="
			<< nameServers[i]
			<< L" index="
			<< i + 1
			<< L" validate=no";

		auto netsh = common::process::ApplicationRunner::StartWithoutConsole(m_netShPath, ss.str());

		validateShellOut(*netsh, timeout);
	}
}

void NetSh::setIpv4DhcpDns(uint32_t interfaceIndex, uint32_t timeout)
{
	//
	// netsh interface ipv4 set dnsservers name="Ethernet 2" source=dhcp
	//
	// Note: we're specifying the interface by index instead.
	//

	std::wstringstream ss;

	ss << L"interface ipv4 set dnsservers name="
		<< interfaceIndex
		<< L" source=dhcp";

	auto netsh = common::process::ApplicationRunner::StartWithoutConsole(m_netShPath, ss.str());

	validateShellOut(*netsh, timeout);
}

void NetSh::setIpv6StaticDns(uint32_t interfaceIndex,
	const std::vector<std::wstring> &nameServers, uint32_t timeout)
{
	//
	// Setting primary and secondary name server requires two invokations:
	//
	// netsh interface ipv6 set dnsservers name="Ethernet 2" source=static address=2001:4860:4860::8888 validate=no
	// netsh interface ipv6 add dnsservers name="Ethernet 2" address=2001:4860:4860::8844 index=2 validate=no
	//
	// Note: we're specifying the interface by index instead.
	//

	if (nameServers.empty())
	{
		THROW_ERROR("Invalid list of name servers (zero length list)");
	}

	{
		std::wstringstream ss;

		ss << L"interface ipv6 set dnsservers name="
			<< interfaceIndex
			<< L" source=static address="
			<< nameServers[0]
			<< L" validate=no";

		auto netsh = common::process::ApplicationRunner::StartWithoutConsole(m_netShPath, ss.str());

		validateShellOut(*netsh, timeout);
	}

	//
	// Set additional name servers.
	//

	for (size_t i = 1; i < nameServers.size(); ++i)
	{
		std::wstringstream ss;

		ss << L"interface ipv6 add dnsservers name="
			<< interfaceIndex
			<< L" address="
			<< nameServers[i]
			<< L" index="
			<< i + 1
			<< L" validate=no";

		auto netsh = common::process::ApplicationRunner::StartWithoutConsole(m_netShPath, ss.str());

		validateShellOut(*netsh, timeout);
	}
}

void NetSh::setIpv6DhcpDns(uint32_t interfaceIndex, uint32_t timeout)
{
	//
	// netsh interface ipv6 set dnsservers name="Ethernet 2" source=dhcp
	//
	// Note: we're specifying the interface by index instead.
	//

	std::wstringstream ss;

	ss << L"interface ipv6 set dnsservers name="
		<< interfaceIndex
		<< L" source=dhcp";

	auto netsh = common::process::ApplicationRunner::StartWithoutConsole(m_netShPath, ss.str());

	validateShellOut(*netsh, timeout);
}

void NetSh::validateShellOut(common::process::ApplicationRunner &netsh, uint32_t timeout)
{
	const uint32_t actualTimeout = (0 == timeout ? 10000 : timeout);

	const auto startTime = GetTickCount64();

	DWORD returnCode;

	if (false == netsh.join(returnCode, actualTimeout))
	{
		ThrowWithDetails("'netsh' did not complete in a timely manner", netsh);
	}

	if (returnCode != 0)
	{
		std::stringstream ss;

		ss << "'netsh' failed the requested operation. Error: " << returnCode;

		ThrowWithDetails(ss.str(), netsh);
	}

	const auto elapsed = static_cast<uint32_t>(GetTickCount64() - startTime);

	if (elapsed > (actualTimeout / 2))
	{
		std::stringstream ss;

		ss << "'netsh' completed successfully, albeit a little slowly. It consumed "
			<< elapsed << " ms of "
			<< actualTimeout << " ms max permitted execution time";

		m_logSink->info(ss.str().c_str());
	}
}
