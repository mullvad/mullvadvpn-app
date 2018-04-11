#pragma once

#include "cli/commands/icommand.h"
#include "cli/util.h"
#include "cli/subcommanddispatcher.h"
#include "libcommon/string.h"

namespace commands::wfpctl
{

class Policy : public ICommand
{
public:

	Policy(MessageSink messageSink);

	std::wstring name() override;
	std::wstring description() override;

	void handleRequest(const std::vector<std::wstring> &arguments) override;

private:

	MessageSink m_messageSink;
	SubcommandDispatcher m_dispatcher;

	using KeyValuePairs = common::string::KeyValuePairs;

	void processConnecting(const KeyValuePairs &arguments);
	void processConnected(const KeyValuePairs &arguments);
	void processReset();
};

}
