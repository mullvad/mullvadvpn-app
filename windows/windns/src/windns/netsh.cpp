#include "stdafx.h"
#include "netsh.h"
#include "libcommon/applicationrunner.h"
#include <sstream>
#include <stdexcept>

namespace
{

void ValidateShellOut(common::ApplicationRunner &netsh)
{
	static const uint32_t TIMEOUT_TWO_SECONDS = 2000;

	DWORD returnCode;

	if (false == netsh.join(returnCode, TIMEOUT_TWO_SECONDS))
	{
		throw std::runtime_error("'netsh' did not complete in a timely manner");
	}

	if (returnCode != 0)
	{
		std::stringstream ss;

		ss << "'netsh' failed the requested operation. Error: " << returnCode;

		throw std::runtime_error(ss.str());
	}
}

} // anonymous namespace

//static
void NetSh::SetIpv4PrimaryDns(uint32_t interfaceIndex, std::wstring server)
{
	//
	// netsh interface ipv4 set dnsservers name="Ethernet 2" source=static address=8.8.8.8 validate=no
	//
	// Note: we're specifying the interface by index instead.
	//

	std::wstringstream ss;

	ss << L"interface ipv4 set dnsservers name="
		<< interfaceIndex
		<< L" source=static address="
		<< server
		<< L" validate=no";

	auto netsh = common::ApplicationRunner::StartWithoutConsole(L"netsh.exe", ss.str());

	ValidateShellOut(*netsh);
}

//static
void NetSh::SetIpv4SecondaryDns(uint32_t interfaceIndex, std::wstring server)
{
	//
	// netsh interface ipv4 add dnsservers name="Ethernet 2" address=8.8.4.4 index=2 validate=no
	//
	// Note: we're specifying the interface by index instead.
	//

	std::wstringstream ss;

	ss << L"interface ipv4 add dnsservers name="
		<< interfaceIndex
		<< L" address="
		<< server
		<< L" index=2 validate=no";

	auto netsh = common::ApplicationRunner::StartWithoutConsole(L"netsh.exe", ss.str());

	ValidateShellOut(*netsh);
}

//static
void NetSh::SetIpv4Dhcp(uint32_t interfaceIndex)
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

	auto netsh = common::ApplicationRunner::StartWithoutConsole(L"netsh.exe", ss.str());

	ValidateShellOut(*netsh);
}

//static
void NetSh::SetIpv6PrimaryDns(uint32_t interfaceIndex, std::wstring server)
{
	//
	// netsh interface ipv6 set dnsservers name="Ethernet 2" source=static address=2001:4860:4860::8888 validate=no
	//
	// Note: we're specifying the interface by index instead.
	//

	std::wstringstream ss;

	ss << L"interface ipv6 set dnsservers name="
		<< interfaceIndex
		<< L" source=static address="
		<< server
		<< L" validate=no";

	auto netsh = common::ApplicationRunner::StartWithoutConsole(L"netsh.exe", ss.str());

	ValidateShellOut(*netsh);
}

//static
void NetSh::SetIpv6SecondaryDns(uint32_t interfaceIndex, std::wstring server)
{
	//
	// netsh interface ipv6 add dnsservers name="Ethernet 2" address=2001:4860:4860::8844 index=2 validate=no
	//
	// Note: we're specifying the interface by index instead.
	//

	std::wstringstream ss;

	ss << L"interface ipv6 add dnsservers name="
		<< interfaceIndex
		<< L"address ="
		<< server
		<< L" index=2 validate=no";

	auto netsh = common::ApplicationRunner::StartWithoutConsole(L"netsh.exe", ss.str());

	ValidateShellOut(*netsh);
}

//static
void NetSh::SetIpv6Dhcp(uint32_t interfaceIndex)
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

	auto netsh = common::ApplicationRunner::StartWithoutConsole(L"netsh.exe", ss.str());

	ValidateShellOut(*netsh);
}
