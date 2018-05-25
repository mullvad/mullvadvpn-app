#include "stdafx.h"
#include "layers.h"
#include "cli/objectproperties.h"
#include "cli/filterengineprovider.h"
#include "cli/propertydecorator.h"
#include "libwfp/objectenumerator.h"

namespace commands::list
{

Layers::Layers(MessageSink messageSink)
	: m_messageSink(messageSink)
{
}

std::wstring Layers::name()
{
	return L"layers";

}

std::wstring Layers::description()
{
	return L"Provides a listing of all layers in the system.";
}

void Layers::handleRequest(const std::vector<std::wstring> &arguments)
{
	if (false == arguments.empty())
	{
		throw std::runtime_error("Unsupported argument(s). Cannot complete request.");
	}

	PrettyPrintOptions options;

	options.indent = 2;
	options.useSeparator = true;

	PropertyDecorator decorator(FilterEngineProvider::Instance().get());

	wfp::ObjectEnumerator::Layers(*FilterEngineProvider::Instance().get(), [&](const FWPM_LAYER0 &layer)
	{
		m_messageSink(L"Layer");

		PrettyPrintProperties(m_messageSink, options, LayerProperties(layer, &decorator));

		return true;
	});
}

}
