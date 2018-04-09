#include "stdafx.h"
#include "providers.h"
#include "cli/objectproperties.h"
#include "cli/filterengineprovider.h"
#include "libwfp/objectenumerator.h"

namespace commands::list
{

Providers::Providers(MessageSink messageSink)
	: m_messageSink(messageSink)
{
}

std::wstring Providers::name()
{
	return L"providers";

}

std::wstring Providers::description()
{
	return L"Provides a listing of all providers in the system.";
}

void Providers::handleRequest(const std::vector<std::wstring> &arguments)
{
	if (false == arguments.empty())
	{
		throw std::runtime_error("Unsupported argument(s). Cannot complete request.");
	}

	PrettyPrintOptions options;

	options.indent = 2;
	options.useSeparator = true;

	wfp::ObjectEnumerator::Providers(*FilterEngineProvider::Instance().get(), [&](const FWPM_PROVIDER0 &provider)
	{
		m_messageSink(L"Provider");

		PrettyPrintProperties(m_messageSink, options, ProviderProperties(provider));

		return true;
	});
}

}
