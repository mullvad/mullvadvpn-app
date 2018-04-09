#include "stdafx.h"
#include "module.h"
#include "cli/util.h"
#include "libcommon/string.h"
#include <sstream>
#include <utility>

namespace modules
{

PropertyList Module::commands()
{
	PropertyList c;

	for (auto &command : m_commands)
	{
		c.add(common::string::Lower(command.second->name()), command.second->description());
	}

	return c;
}

void Module::handleRequest(const std::vector<std::wstring> &request)
{
	//
	// The request has the form of:
	//
	// [0] command
	// [1] arg1
	// [2] arg2
	// ...
	//

	if (request.empty())
	{
		std::wstringstream ss;

		ss << L"Command missing. Try 'help " << m_name << "'.";

		throw std::runtime_error(common::string::ToAnsi(ss.str()));
	}

	auto wanted = common::string::Lower(request[0]);
	auto found = m_commands.find(wanted);

	if (found == m_commands.end())
	{
		std::wstringstream ss;

		ss << L"Module '" << m_name << "' doesn't support the command '" << request[0] << "'.";

		throw std::runtime_error(common::string::ToAnsi(ss.str()));
	}

	auto args = request;

	args.erase(args.begin());

	found->second->handleRequest(args);
}

void Module::addCommand(std::unique_ptr<commands::ICommand> command)
{
	m_commands.insert(std::make_pair(
		common::string::Lower(command->name()),
		std::move(command)
	));
}

}
