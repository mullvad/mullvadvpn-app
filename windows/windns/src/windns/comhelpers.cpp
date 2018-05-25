#include "stdafx.h"
#include "comhelpers.h"
#include <algorithm>

_variant_t ComGetProperty(CComPtr<IWbemClassObject> obj, const std::wstring &name)
{
	_variant_t val;

	const auto status = obj->Get(name.c_str(), 0, &val, nullptr, nullptr);

	VALIDATE_COM(status, "Retrieve COM property value");

	return val;
}

_variant_t ComGetPropertyAlways(CComPtr<IWbemClassObject> obj, const std::wstring &name)
{
	auto val = ComGetProperty(obj, name);

	if (VT_EMPTY == V_VT(&val) || VT_NULL == V_VT(&val))
	{
		throw std::runtime_error("A required COM property value is empty.");
	}

	return val;
}

std::wstring ComConvertString(BSTR src)
{
	return std::wstring(src, SysStringLen(src));
}

std::vector<std::wstring> ComConvertStringArray(SAFEARRAY *src)
{
	CComSafeArray<BSTR> safeArray(src);

	std::vector<std::wstring> result;
	result.reserve(safeArray.GetCount());

	for (ULONG i = 0; i < safeArray.GetCount(); ++i)
	{
		result.emplace_back(ComConvertString(safeArray.GetAt(i)));
	}

	return result;
}

CComSafeArray<BSTR> ComConvertIntoStringArray(const std::vector<std::wstring> &src)
{
	CComSafeArray<BSTR> result;

	std::for_each(src.begin(), src.end(), [&](const std::wstring &str)
	{
		result.Add(_bstr_t(str.c_str()));
	});

	return result;
}

_variant_t ComPackageStringArray(CComSafeArray<BSTR> &src)
{
	VARIANT v;

	V_VT(&v) = VT_ARRAY | VT_BSTR;
	V_ARRAY(&v) = src.Detach();

	_variant_t vv;

	vv.Attach(v);

	return vv;
}