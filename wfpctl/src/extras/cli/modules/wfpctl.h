#pragma once

#include "module.h"
#include "cli/util.h"
#include "cli/commands/wfpctl/init.h"
#include "cli/commands/wfpctl/deinit.h"
#include "cli/commands/wfpctl/policy.h"

namespace modules
{

class Wfpctl : public Module
{
public:

	Wfpctl(MessageSink messageSink)
		: Module(L"wfpctl", L"Exercise functionality provided by \"wfpctl.dll\".")
	{
		addCommand(std::make_unique<commands::wfpctl::Init>(messageSink));
		addCommand(std::make_unique<commands::wfpctl::Deinit>(messageSink));
		addCommand(std::make_unique<commands::wfpctl::Policy>(messageSink));
	}
};

}
