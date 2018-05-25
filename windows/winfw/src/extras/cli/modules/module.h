#pragma once

#include "imodule.h"
#include "cli/commands/icommand.h"
#include <map>
#include <memory>

namespace modules
{

class Module : public IModule
{
public:

	Module(const std::wstring &name, const std::wstring &description)
		: m_name(name)
		, m_description(description)
	{
	}

	std::wstring name() override
	{
		return m_name;
	}

	std::wstring description() override
	{
		return m_description;
	}

	// Collect name and description from commands.
	PropertyList commands() override;

	// Identify requested command and dispatch to it.
	void handleRequest(const std::vector<std::wstring> &request) override;

	void addCommand(std::unique_ptr<commands::ICommand> command);

private:

	Module(const Module &);
	Module &operator=(const Module &);

	std::wstring m_name;
	std::wstring m_description;

	std::map<std::wstring, std::unique_ptr<commands::ICommand> > m_commands;
};

}
