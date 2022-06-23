#include "stdafx.h"
#include <sstream>
#include <algorithm>
#include "interfaceutils.h"
#include <libcommon/error.h>
#include <libcommon/string.h>

namespace shared::network
{

InterfaceUtils::NetworkAdapter::NetworkAdapter(
	const std::shared_ptr<std::vector<uint8_t>> addressesBuffer,
	const IP_ADAPTER_ADDRESSES &entry
)
	: m_addressesBuffer(addressesBuffer)
	, m_entry(entry)
{
	m_guid = common::string::ToWide(entry.AdapterName);
	m_alias = entry.FriendlyName;
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

	for (auto it = addresses; nullptr != it; it = it->Next)
	{
		adapters.emplace(NetworkAdapter(buffer, *it));
	}

	return adapters;
}

}
