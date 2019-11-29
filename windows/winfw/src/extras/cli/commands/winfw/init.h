#pragma once

#include "cli/commands/icommand.h"
#include "cli/util.h"
#include "winfw/winfw.h"

namespace commands::winfw
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

	static void WINFW_API ErrorForwarder(MULLVAD_LOG_LEVEL level, const char *errorMessage, void *context);
};

}
