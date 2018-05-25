#include "stdafx.h"
#include "sublayers.h"
#include "cli/objectproperties.h"
#include "cli/filterengineprovider.h"
#include "cli/propertydecorator.h"
#include "libwfp/objectenumerator.h"

namespace commands::list
{

Sublayers::Sublayers(MessageSink messageSink)
	: m_messageSink(messageSink)
{
}

std::wstring Sublayers::name()
{
	return L"sublayers";

}

std::wstring Sublayers::description()
{
	return L"Provides a listing of all sublayers in the system.";
}

void Sublayers::handleRequest(const std::vector<std::wstring> &arguments)
{
	if (false == arguments.empty())
	{
		throw std::runtime_error("Unsupported argument(s). Cannot complete request.");
	}

	PrettyPrintOptions options;

	options.indent = 2;
	options.useSeparator = true;

	PropertyDecorator decorator(FilterEngineProvider::Instance().get());

	wfp::ObjectEnumerator::Sublayers(*FilterEngineProvider::Instance().get(), [&](const FWPM_SUBLAYER0 &sublayer)
	{
		m_messageSink(L"Sublayer");

		PrettyPrintProperties(m_messageSink, options, SublayerProperties(sublayer, &decorator));

		return true;
	});
}

}
