#pragma once

#include "module.h"
#include "cli/util.h"
#include "cli/commands/list/sessions.h"
#include "cli/commands/list/providers.h"
#include "cli/commands/list/events.h"
#include "cli/commands/list/filters.h"
#include "cli/commands/list/layers.h"
#include "cli/commands/list/providercontexts.h"
#include "cli/commands/list/sublayers.h"

namespace modules
{

class List : public Module
{
public:

	List(MessageSink messageSink)
		: Module(L"list", L"List various objects in the WFP universe.")
	{
		addCommand(std::make_unique<commands::list::Sessions>(messageSink));
		addCommand(std::make_unique<commands::list::Providers>(messageSink));
		addCommand(std::make_unique<commands::list::Events>(messageSink));
		addCommand(std::make_unique<commands::list::Filters>(messageSink));
		addCommand(std::make_unique<commands::list::Layers>(messageSink));
		addCommand(std::make_unique<commands::list::ProviderContexts>(messageSink));
		addCommand(std::make_unique<commands::list::Sublayers>(messageSink));
	}
};

}
