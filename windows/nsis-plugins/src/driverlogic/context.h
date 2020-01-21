#pragma once

#include <set>
#include <string>
#include <optional>

class Context
{
public:

	Context()
	{
	}

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

	enum class BaselineStatus
	{
		NO_TAP_ADAPTERS_PRESENT,
		SOME_TAP_ADAPTERS_PRESENT,
		MULLVAD_ADAPTER_PRESENT
	};

	BaselineStatus establishBaseline();

	void recordCurrentState();

	//
	// Restore TAP aliases to baseline state
	//
	void rollbackTapAliases();

	//
	// Identify a single new TAP adapter
	//
	NetworkAdapter getNewAdapter();

	enum class DeletionResult
	{
		NO_REMAINING_TAP_ADAPTERS,
		SOME_REMAINING_TAP_ADAPTERS
	};

	static DeletionResult DeleteOldMullvadAdapter();

private:

	static std::optional<NetworkAdapter> FindMullvadAdapter(const std::set<NetworkAdapter> &tapAdapters);

	std::set<NetworkAdapter> m_baseline;
	std::set<NetworkAdapter> m_currentState;
};
