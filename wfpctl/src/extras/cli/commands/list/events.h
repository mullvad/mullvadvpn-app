#pragma once

#include "cli/commands/icommand.h"
#include "cli/util.h"

namespace commands::list
{

class Events : public ICommand
{
public:

	Events(MessageSink messageSink);

	std::wstring name() override;
	std::wstring description() override;

	void handleRequest(const std::vector<std::wstring> &arguments) override;

private:

	MessageSink m_messageSink;
};

}
