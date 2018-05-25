#include "stdafx.h"
#include "m_events.h"
#include "cli/objectproperties.h"
#include "cli/propertydecorator.h"
#include "cli/filterengineprovider.h"
#include "libwfp/objectmonitor.h"
#include <conio.h>

namespace commands::monitor
{

Events::Events(MessageSink messageSink)
	: m_messageSink(messageSink)
{
}

std::wstring Events::name()
{
	return L"events";

}

std::wstring Events::description()
{
	return L"Provides monitoring of drop/allow events.";
}

void Events::handleRequest(const std::vector<std::wstring> &arguments)
{
	if (false == arguments.empty())
	{
		throw std::runtime_error("Unsupported argument(s). Cannot complete request.");
	}

	wfp::ObjectMonitor objectMonitor(FilterEngineProvider::Instance().get());

	objectMonitor.monitorEvents(std::bind(&Events::eventCallback, this, std::placeholders::_1));

	m_messageSink(L"Successfully enabled monitor. Press any key to abort monitoring.");

	//
	// This assumes we're in a console environment, but the alternative is to have
	// the command (this class) communicate to the outside world that we're in
	// a monitoring state.
	//
	// Hence, this is sufficient for now.
	//

	_getwch();

	objectMonitor.monitorEventsStop();
}

void Events::eventCallback(const FWPM_NET_EVENT1 &event)
{
	m_messageSink(L"Event");

	PrettyPrintOptions options;

	options.indent = 2;
	options.useSeparator = true;

	PropertyDecorator decorator(FilterEngineProvider::Instance().get());

	PrettyPrintProperties(m_messageSink, options, EventProperties(event, &decorator));
}

}
