#include "stdafx.h"
#include "providercontexts.h"
#include "cli/objectproperties.h"
#include "cli/filterengineprovider.h"
#include "cli/propertydecorator.h"
#include "libwfp/objectenumerator.h"

namespace commands::list
{

ProviderContexts::ProviderContexts(MessageSink messageSink)
	: m_messageSink(messageSink)
{
}

std::wstring ProviderContexts::name()
{
	return L"providercontexts";

}

std::wstring ProviderContexts::description()
{
	return L"Provides a listing of all provider contexts in the system.";
}

void ProviderContexts::handleRequest(const std::vector<std::wstring> &arguments)
{
	if (false == arguments.empty())
	{
		throw std::runtime_error("Unsupported argument(s). Cannot complete request.");
	}

	PrettyPrintOptions options;

	options.indent = 2;
	options.useSeparator = true;

	PropertyDecorator decorator(FilterEngineProvider::Instance().get());

	wfp::ObjectEnumerator::ProviderContexts(*FilterEngineProvider::Instance().get(), [&](const FWPM_PROVIDER_CONTEXT0 &context)
	{
		m_messageSink(L"Provider context");

		PrettyPrintProperties(m_messageSink, options, ProviderContextProperties(context, &decorator));

		return true;
	});
}

}
