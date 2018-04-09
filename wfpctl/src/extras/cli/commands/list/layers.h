#pragma once

#include "cli/commands/icommand.h"
#include "cli/util.h"

namespace commands::list
{

class Layers : public ICommand
{
public:

	Layers(MessageSink messageSink);

	std::wstring name() override;
	std::wstring description() override;

	void handleRequest(const std::vector<std::wstring> &arguments) override;

private:

	MessageSink m_messageSink;
};

}
