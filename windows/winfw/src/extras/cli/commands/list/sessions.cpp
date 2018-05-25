#include "stdafx.h"
#include "sessions.h"
#include "cli/objectproperties.h"
#include "cli/filterengineprovider.h"
#include "libwfp/objectenumerator.h"

namespace commands::list
{

Sessions::Sessions(MessageSink messageSink)
	: m_messageSink(messageSink)
{
}

std::wstring Sessions::name()
{
	return L"sessions";

}

std::wstring Sessions::description()
{
	return L"Provides a listing of all active sessions in the system.";
}

void Sessions::handleRequest(const std::vector<std::wstring> &arguments)
{
	if (false == arguments.empty())
	{
		throw std::runtime_error("Unsupported argument(s). Cannot complete request.");
	}

	PrettyPrintOptions options;

	options.indent = 2;
	options.useSeparator = true;

	wfp::ObjectEnumerator::Sessions(*FilterEngineProvider::Instance().get(), [&](const FWPM_SESSION0 &session)
	{
		m_messageSink(L"Session");

		PrettyPrintProperties(m_messageSink, options, SessionProperties(session));

		return true;
	});
}

}
