#include "stdafx.h"
#include "propertydecorator.h"
#include "libwfp/objectexplorer.h"
#include "libcommon/string.h"
#include "inlineformatter.h"
#include <cwchar>

namespace detail
{

std::wstring Format(const wchar_t *in_name, const wchar_t *in_desc)
{
	auto haveName = (nullptr != in_name && 0 != wcslen(in_name));
	auto haveDesc = (nullptr != in_desc && 0 != wcslen(in_desc));

	std::wstring name = (haveName ? in_name : L"n/a");
	std::wstring desc = (haveDesc ? common::string::Summary(in_desc, 50) : L"n/a");

	return (InlineFormatter() << L"[" << name << L", " << desc << L"]").str();
}

} // namespace detail

PropertyDecorator::PropertyDecorator(std::shared_ptr<wfp::FilterEngine> engine)
	: m_engine(engine)
{
}

std::wstring PropertyDecorator::FilterDecoration(UINT64 id)
{
	std::wstring brief = L"[n/a]";

	wfp::ObjectExplorer::GetFilter(*m_engine, id, [&brief](const FWPM_FILTER0 &filter)
	{
		brief = detail::Format(filter.displayData.name, filter.displayData.description);
		return true;
	});

	return brief;
}

std::wstring PropertyDecorator::LayerDecoration(UINT16 id)
{
	std::wstring brief = L"[n/a]";

	wfp::ObjectExplorer::GetLayer(*m_engine, id, [&brief](const FWPM_LAYER0 &layer)
	{
		brief = detail::Format(layer.displayData.name, layer.displayData.description);
		return true;
	});

	return brief;
}

std::wstring PropertyDecorator::LayerDecoration(const GUID &key)
{
	std::wstring brief = L"[n/a]";

	wfp::ObjectExplorer::GetLayer(*m_engine, key, [&brief](const FWPM_LAYER0 &layer)
	{
		brief = detail::Format(layer.displayData.name, layer.displayData.description);
		return true;
	});

	return brief;
}

std::wstring PropertyDecorator::ProviderDecoration(const GUID &key)
{
	std::wstring brief = L"[n/a]";

	wfp::ObjectExplorer::GetProvider(*m_engine, key, [&brief](const FWPM_PROVIDER0 &provider)
	{
		brief = detail::Format(provider.displayData.name, provider.displayData.description);
		return true;
	});

	return brief;
}

std::wstring PropertyDecorator::SublayerDecoration(const GUID &key)
{
	std::wstring brief = L"[n/a]";

	wfp::ObjectExplorer::GetSublayer(*m_engine, key, [&brief](const FWPM_SUBLAYER0 &sublayer)
	{
		brief = detail::Format(sublayer.displayData.name, sublayer.displayData.description);
		return true;
	});

	return brief;
}
