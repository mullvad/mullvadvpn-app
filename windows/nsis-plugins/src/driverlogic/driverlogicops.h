#pragma once

#include <set>
#include <string>
#include <optional>

namespace driverlogic
{

struct NetworkAdapter
{
	std::wstring guid;
	std::wstring name;
	std::wstring alias;
	std::wstring deviceInstanceId;

	NetworkAdapter(std::wstring guid, std::wstring name, std::wstring alias, std::wstring deviceInstanceId)
		: guid(guid)
		, name(name)
		, alias(alias)
		, deviceInstanceId(deviceInstanceId)
	{
	}

	bool operator<(const NetworkAdapter &rhs) const
	{
		return _wcsicmp(deviceInstanceId.c_str(), rhs.deviceInstanceId.c_str()) < 0;
	}
};

//
// Identify the Mullvad TAP adapter
//
NetworkAdapter GetAdapter();

enum class DeletionResult
{
	NO_REMAINING_TAP_ADAPTERS,
	SOME_REMAINING_TAP_ADAPTERS
};

DeletionResult DeleteOldMullvadAdapter();

std::optional<NetworkAdapter> FindMullvadAdapter(const std::set<NetworkAdapter> &tapAdapters);

}
