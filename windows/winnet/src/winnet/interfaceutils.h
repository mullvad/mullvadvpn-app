#pragma once

#include <string>
#include <set>
#include <mutex>

class InterfaceUtils
{
	InterfaceUtils() = delete;

	static std::wstring m_alias;
	static std::mutex m_mutex;

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
};
