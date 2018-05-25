#include "stdafx.h"
#include "subcommanddispatcher.h"
#include "libcommon/string.h"
#include <sstream>
#include <stdexcept>
#include <utility>

void SubcommandDispatcher::addSubcommand(const std::wstring &command, Handler handler)
{
	m_commands.insert(std::make_pair(command, handler));
}

void SubcommandDispatcher::dispatch(const std::wstring &command, const std::vector<std::wstring> &arguments)
{
	auto selectedCommand = m_commands.find(command);

	if (m_commands.end() == selectedCommand)
	{
		std::wstringstream ss;

		ss << L"Unsupported subcommand '" << command << "'. Cannot complete request.";

		throw std::runtime_error(common::string::ToAnsi(ss.str()).c_str());
	}

	selectedCommand->second(common::string::SplitKeyValuePairs(arguments));
}
