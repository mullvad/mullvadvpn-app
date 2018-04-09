#pragma once

#include "cli/commands/icommand.h"
#include "cli/util.h"
#include "wfpctl/wfpctl.h"

namespace commands::wfpctl
{

class Init : public ICommand
{
public:

	Init(MessageSink messageSink);

	std::wstring name() override;
	std::wstring description() override;

	void handleRequest(const std::vector<std::wstring> &arguments) override;

private:

	MessageSink m_messageSink;

	static void WFPCTL_API ErrorForwarder(const char *errorMessage, void *context);
};

}
