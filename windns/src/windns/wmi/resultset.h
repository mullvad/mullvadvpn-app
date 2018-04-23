#pragma once

#include <string>
#include <atlbase.h>
#include <wbemidl.h>

namespace wmi
{

class ResultSet
{
public:

	ResultSet(CComPtr<IEnumWbemClassObject> rs);

	ResultSet(const ResultSet &) = delete;
	ResultSet &operator=(const ResultSet &) = delete;
	ResultSet(ResultSet &&) = default;
	ResultSet &operator=(ResultSet &&) = default;

	bool advance();

	CComPtr<IWbemClassObject> result();

private:

	CComPtr<IEnumWbemClassObject> m_resultset;
	CComPtr<IWbemClassObject> m_result;
};

}
