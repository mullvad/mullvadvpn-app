#pragma once

#include "module.h"
#include "cli/util.h"
#include "cli/commands/winfw/init.h"
#include "cli/commands/winfw/deinit.h"
#include "cli/commands/winfw/policy.h"

namespace modules
{

class WinFw : public Module
{
public:

	WinFw(MessageSink messageSink)
		: Module(L"winfw", L"Exercise functionality provided by \"winfw.dll\".")
	{
		addCommand(std::make_unique<commands::winfw::Init>(messageSink));
		addCommand(std::make_unique<commands::winfw::Deinit>(messageSink));
		addCommand(std::make_unique<commands::winfw::Policy>(messageSink));
	}
};

}
