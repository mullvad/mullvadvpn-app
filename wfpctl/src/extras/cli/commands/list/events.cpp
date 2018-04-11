#include "stdafx.h"
#include "events.h"
#include "cli/objectproperties.h"
#include "cli/filterengineprovider.h"
#include "cli/propertydecorator.h"
#include "libwfp/objectenumerator.h"

namespace commands::list
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
	return L"Provides a listing of all recent events in the system.";
}

void Events::handleRequest(const std::vector<std::wstring> &arguments)
{
	if (false == arguments.empty())
	{
		throw std::runtime_error("Unsupported argument(s). Cannot complete request.");
	}

	PrettyPrintOptions options;

	options.indent = 2;
	options.useSeparator = true;

	PropertyDecorator decorator(FilterEngineProvider::Instance().get());

	wfp::ObjectEnumerator::Events(*FilterEngineProvider::Instance().get(), [&](const FWPM_NET_EVENT0 &event)
	{
		m_messageSink(L"Event");

		PrettyPrintProperties(m_messageSink, options, EventProperties(event, &decorator));

		return true;
	});
}

}
