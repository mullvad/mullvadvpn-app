#pragma once

#include <set>
#include <string>

class Context
{
public:

	struct VirtualNic
	{
		std::wstring node;
		std::wstring name;
		std::wstring alias;

		bool operator<(const VirtualNic &rhs) const
		{
			return _wcsicmp(node.c_str(), rhs.node.c_str()) < 0;
		}
	};

	enum class BaselineStatus
	{
		NO_INTERFACES_PRESENT,
		SOME_INTERFACES_PRESENT,
		MULLVAD_INTERFACE_PRESENT
	};

	//
	// Invoke with the output from "tapinstall hwids tap0901"
	//
	BaselineStatus establishBaseline(const std::wstring &textBlock);

	//
	// Invoke with the output from "tapinstall hwids tap0901"
	//
	void recordCurrentState(const std::wstring &textBlock);

	//
	// Identify a single new interface
	//
	VirtualNic getNewAdapter();

private:

	static std::set<VirtualNic> ParseVirtualNics(const std::wstring &textBlock);
	static std::wstring GetNicAlias(const std::wstring &node, const std::wstring &name);

	std::set<VirtualNic> m_baseline;
	std::set<VirtualNic> m_currentState;
};
