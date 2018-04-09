#pragma once

#include "module.h"
#include "cli/util.h"
#include "cli/commands/monitor/m_events.h"

namespace modules
{

class Monitor : public Module
{
public:

	Monitor(MessageSink messageSink)
		: Module(L"monitor", L"Real-time monitoring of events and object creation/deletion in WFP.")
	{
		addCommand(std::make_unique<commands::monitor::Events>(messageSink));
	}
};

}
