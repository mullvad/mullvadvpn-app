#pragma once

#include <set>
#include <string>

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

	enum class BaselineStatus
	{
		NO_TAP_ADAPTERS_PRESENT,
		SOME_TAP_ADAPTERS_PRESENT,
		MULLVAD_ADAPTER_PRESENT
	};

	BaselineStatus establishBaseline();

	void recordCurrentState();

	//
	// Identify a single new TAP adapter
	//
	NetworkAdapter getNewAdapter();

	std::set<NetworkAdapter> getTapAdapters();
private:

	std::set<NetworkAdapter> m_baseline;
	std::set<NetworkAdapter> m_currentState;
};
