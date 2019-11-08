#pragma once

#include <string>
#include <set>
#include <vector>

// Secret include order to get most common networking structs/apis
// And avoiding compilation errors
#include <winsock2.h>
#include <windows.h>
#include <ws2def.h>
#include <ws2ipdef.h>
#include <iphlpapi.h>
#include <netioapi.h>
// end

class InterfaceUtils
{
	InterfaceUtils() = delete;

public:

	struct NetworkAdapter
	{
		std::wstring guid;
		std::wstring name;
		std::wstring alias;

		NetworkAdapter(std::wstring _guid, std::wstring _name, std::wstring _alias)
			: guid(_guid)
			, name(_name)
			, alias(_alias)
		{
		}

		bool operator<(const NetworkAdapter &rhs) const
		{
			return _wcsicmp(guid.c_str(), rhs.guid.c_str()) < 0;
		}
	};

	static std::set<NetworkAdapter> GetAllAdapters();
	static std::set<NetworkAdapter> GetTapAdapters(const std::set<NetworkAdapter> &adapters);

	//
	// Determines alias of primary TAP adapter.
	//
	static std::wstring GetTapInterfaceAlias();

	static void AddDeviceIpAddresses(NET_LUID device, const std::vector<SOCKADDR_INET> &addresses);
};
