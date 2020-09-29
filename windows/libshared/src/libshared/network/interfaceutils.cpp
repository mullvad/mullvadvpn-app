#include "stdafx.h"
#include <sstream>
#include <algorithm>
#include "interfaceutils.h"
#include <libcommon/error.h>
#include <libcommon/string.h>

namespace
{

// Interface description substrings found for virtual adapters.
const wchar_t *TUNNEL_INTERFACE_DESCS[] = {
	L"WireGuard",
	L"Wintun",
	L"Mullvad"
};

} // anonymous namespace

namespace shared::network
{

InterfaceUtils::NetworkAdapter::NetworkAdapter(
	const common::network::Nci &nci,
	const std::shared_ptr<std::vector<uint8_t>> addressesBuffer,
	const IP_ADAPTER_ADDRESSES &entry
)
	: m_addressesBuffer(addressesBuffer)
	, m_entry(entry)
{
	m_guid = common::string::ToWide(entry.AdapterName);

	try
	{
		//
		// FIXME:
		// Work around incorrect alias sometimes
		// being returned on Windows 8.
		//
		// Steps to reproduce:
		// 1. Install NDIS 6 TAP driver v9.00.00.21.
		// 2. Update driver to v9.24.2.601.
		// 3. Rename TAP adapter.
		//
		// GetAdaptersAddresses() returns a generic name
		// for the *first* adapter instead of the correct
		// one, whereas ConvertInterfaceAliasToLuid() and
		// ConvertInterfaceLuidToAlias() yield correct values.
		//

		IID guidObj = { 0 };
		if (S_OK != IIDFromString(&m_guid[0], &guidObj))
		{
			THROW_ERROR("IIDFromString() failed");
		}

		m_alias = nci.getConnectionName(guidObj);
	}
	catch (const std::exception &)
	{
		m_alias = entry.FriendlyName;
	}

	m_name = entry.Description;
}

//static
std::set<InterfaceUtils::NetworkAdapter> InterfaceUtils::GetAllAdapters(ULONG family, ULONG flags)
{
	ULONG bufferSize = 0;

	auto status = GetAdaptersAddresses(family, flags, nullptr, nullptr, &bufferSize);

	if (ERROR_BUFFER_OVERFLOW != status)
	{
		THROW_WINDOWS_ERROR(status, "Probe for adapter listing buffer size");
	}

	// Memory is cheap, this avoids a looping construct.
	bufferSize *= 2;

	auto buffer = std::make_shared<std::vector<uint8_t>>(bufferSize);
	auto addresses = reinterpret_cast<PIP_ADAPTER_ADDRESSES>(&(*buffer)[0]);

	status = GetAdaptersAddresses(family, flags, nullptr, addresses, &bufferSize);

	if (ERROR_SUCCESS != status)
	{
		THROW_WINDOWS_ERROR(status, "Retrieve adapter listing");
	}

	std::set<NetworkAdapter> adapters;

	common::network::Nci nci;

	for (auto it = addresses; nullptr != it; it = it->Next)
	{
		adapters.emplace(NetworkAdapter(nci, buffer, *it));
	}

	return adapters;
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

		const auto status = CreateUnicastIpAddressEntry(&row);

		if (NO_ERROR != status)
		{
			THROW_WINDOWS_ERROR(status, "Assign IP address on network interface");
		}
	}
}

//static
std::set<InterfaceUtils::NetworkAdapter>
InterfaceUtils::GetVirtualAdapters(const std::set<NetworkAdapter>& adapters)
{
	std::set<NetworkAdapter> virtualAdapters;

	for (const auto& adapter : adapters)
	{
		for (size_t i = 0; i < ARRAYSIZE(TUNNEL_INTERFACE_DESCS); i++)
		{
			if (nullptr != wcsstr(adapter.raw().Description, TUNNEL_INTERFACE_DESCS[i]))
			{
				virtualAdapters.insert(adapter);
			}
		}
	}

	return virtualAdapters;
}

//static
std::wstring InterfaceUtils::GetInterfaceAlias()
{
	//
	// Look for virtual adapter with alias "Mullvad".
	//

	using shared::network::InterfaceUtils;

	auto adapters = InterfaceUtils::GetVirtualAdapters(InterfaceUtils::GetAllAdapters(
		AF_INET,
		GAA_FLAG_SKIP_UNICAST | GAA_FLAG_SKIP_ANYCAST | GAA_FLAG_SKIP_MULTICAST
	));

	auto findByAlias = [](const std::set<InterfaceUtils::NetworkAdapter>& adapters, const std::wstring& alias)
	{
		const auto it = std::find_if(adapters.begin(), adapters.end(), [&alias](const InterfaceUtils::NetworkAdapter& candidate)
		{
			return 0 == _wcsicmp(candidate.alias().c_str(), alias.c_str());
		});

		return it != adapters.end();
	};

	static const wchar_t baseAlias[] = L"Mullvad";

	if (findByAlias(adapters, baseAlias))
	{
		return baseAlias;
	}

	//
	// Look for virtual adapter with alias "Mullvad-1", "Mullvad-2", etc.
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

	THROW_ERROR("Unable to find virtual adapter");
}

}
