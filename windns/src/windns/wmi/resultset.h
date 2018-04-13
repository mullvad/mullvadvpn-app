#pragma once

#include <string>
#include <atlbase.h>
#include <wbemidl.h>
#include <comutil.h>

namespace wmi
{

class ResultSet
{
public:

	ResultSet(CComPtr<IEnumWbemClassObject> rs);

	ResultSet(ResultSet &&) = default;
	ResultSet &operator=(ResultSet &&) = default;

	bool advance();

	_variant_t getResultProperty(const std::wstring &name);

	CComPtr<IWbemClassObject> getResult();

private:

	ResultSet(const ResultSet &) = delete;
	ResultSet &operator=(const ResultSet &) = delete;

	CComPtr<IEnumWbemClassObject> m_resultset;
	CComPtr<IWbemClassObject> m_result;
};

}
