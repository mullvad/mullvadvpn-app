#include "stdafx.h"
#include "interfaceutils.h"
#include "libcommon/error.h"
#include "libcommon/string.h"
#include <cstdint>
#include <algorithm>

//static
std::set<InterfaceUtils::NetworkAdapter> InterfaceUtils::GetAllAdapters()
{
	ULONG bufferSize = 0;

	const ULONG flags = GAA_FLAG_SKIP_UNICAST | GAA_FLAG_SKIP_ANYCAST | GAA_FLAG_SKIP_MULTICAST | GAA_FLAG_SKIP_DNS_SERVER;

	auto status = GetAdaptersAddresses(AF_INET, flags, nullptr, nullptr, &bufferSize);

	THROW_UNLESS(ERROR_BUFFER_OVERFLOW, status, "Probe for adapter listing buffer size");

	// Memory is cheap, this avoids a looping construct.
	bufferSize *= 2;

	std::vector<uint8_t> buffer(bufferSize);

	status = GetAdaptersAddresses(AF_INET, flags, nullptr,
		reinterpret_cast<PIP_ADAPTER_ADDRESSES>(&buffer[0]), &bufferSize);

	THROW_UNLESS(ERROR_SUCCESS, status, "Retrieve adapter listing");

	std::set<NetworkAdapter> adapters;

	for (auto it = (PIP_ADAPTER_ADDRESSES)&buffer[0]; nullptr != it; it = it->Next)
	{
		adapters.emplace(NetworkAdapter(common::string::ToWide(it->AdapterName),
			it->Description, it->FriendlyName));
	}

	return adapters;
}

//static
std::set<InterfaceUtils::NetworkAdapter>
InterfaceUtils::GetTapAdapters(const std::set<InterfaceUtils::NetworkAdapter> &adapters)
{
	std::set<NetworkAdapter> tapAdapters;

	for (const auto &adapter : adapters)
	{
		static const wchar_t name[] = L"TAP-Windows Adapter V9";

		//
		// Compare partial name, because once you start having more TAP adapters
		// they're named "TAP-Windows Adapter V9 #2" and so on.
		//

		if (0 == adapter.name.compare(0, _countof(name) - 1, name))
		{
			tapAdapters.insert(adapter);
		}
	}

	return tapAdapters;
}

//static
std::wstring InterfaceUtils::GetTapInterfaceAlias()
{
	//
	// Look for TAP adapter with alias "Mullvad".
	//

	auto adapters = GetTapAdapters(GetAllAdapters());

	auto findByAlias = [](const std::set<NetworkAdapter> &adapters, const std::wstring &alias)
	{
		const auto it = std::find_if(adapters.begin(), adapters.end(), [&alias](const NetworkAdapter &candidate)
		{
			return 0 == _wcsicmp(candidate.alias.c_str(), alias.c_str());
		});

		return it != adapters.end();
	};

	static const wchar_t baseAlias[] = L"Mullvad";

	if (findByAlias(adapters, baseAlias))
	{
		return baseAlias;
	}

	//
	// Look for TAP adapter with alias "Mullvad-1", "Mullvad-2", etc.
	//

	for (auto i = 0; i < 10; ++i)
	{
		std::wstringstream ss;

		ss << baseAlias << L"-" << i;

		const auto alias = ss.str();

		if (findByAlias(adapters, alias))
		{
			return alias;
		}
	}

	throw std::runtime_error("Unable to find TAP adapter");
}

//static
void InterfaceUtils::AddDeviceIpAddresses(NET_LUID device, const std::vector<SOCKADDR_INET> &addresses)
{
	for (const auto &address : addresses)
	{
		MIB_UNICASTIPADDRESS_ROW row;
		InitializeUnicastIpAddressEntry(&row);

		row.InterfaceLuid = device;
		row.Address = address;

		THROW_UNLESS(NO_ERROR, CreateUnicastIpAddressEntry(&row), "Assign IP address on network interface");
	}
}
