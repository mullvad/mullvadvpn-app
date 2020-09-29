#pragma once

#include <string>
#include <set>
#include <vector>
#include <memory>
#include <cstdint>

// Secret include order to get most common networking structs/apis
// And avoiding compilation errors
#include <winsock2.h>
#include <windows.h>
#include <ws2def.h>
#include <ws2ipdef.h>
#include <iphlpapi.h>
#include <netioapi.h>
// end

#include <libcommon/network/nci.h>

namespace shared::network
{

class InterfaceUtils
{
	InterfaceUtils() = delete;

public:

	class NetworkAdapter
	{

	public:

		const std::wstring &guid() const { return m_guid; }
		const std::wstring &name() const { return m_name; }
		const std::wstring &alias() const { return m_alias; }

		bool operator<(const NetworkAdapter &rhs) const
		{
			return _wcsicmp(m_guid.c_str(), rhs.m_guid.c_str()) < 0;
		}

		const IP_ADAPTER_ADDRESSES &raw() const
		{
			return m_entry;
		}

	private:

		NetworkAdapter(
			const common::network::Nci &nci,
			const std::shared_ptr<std::vector<uint8_t>> addressesBuffer,
			const IP_ADAPTER_ADDRESSES &entry
		);

		friend class InterfaceUtils;

		const IP_ADAPTER_ADDRESSES &m_entry;
		std::shared_ptr<std::vector<uint8_t>> m_addressesBuffer;

		std::wstring m_guid;
		std::wstring m_name;
		std::wstring m_alias;
	};

	static std::set<NetworkAdapter> GetAllAdapters(ULONG family, ULONG flags);

	static void AddDeviceIpAddresses(NET_LUID device, const std::vector<SOCKADDR_INET> &addresses);

	static std::set<NetworkAdapter> GetVirtualAdapters(const std::set<NetworkAdapter> &adapters);

	//
	// Determines alias of primary virtual adapter.
	//
	static std::wstring GetInterfaceAlias();
};

}
