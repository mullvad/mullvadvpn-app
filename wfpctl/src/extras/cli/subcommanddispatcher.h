#pragma once

#include "libcommon/string.h"
#include <functional>
#include <string>
#include <unordered_map>

class SubcommandDispatcher
{
	typedef std::function<void(const common::string::KeyValuePairs &)> Handler;

public:

	void addSubcommand(const std::wstring &command, Handler handler);
	void dispatch(const std::wstring &command, const std::vector<std::wstring> &arguments);

private:

	std::unordered_map<std::wstring, Handler> m_commands;
};
