#pragma once

#include "iconnection.h"
#include <string>
#define _WIN32_DCOM
#include <windows.h>
#include <atlbase.h>
#include <comutil.h>

namespace wmi
{

class Connection : public IConnection
{
public:

	enum class Namespace
	{
		Default,
		Cimv2
	};

	explicit Connection(Namespace ns);

	ResultSet Query(const wchar_t *query) override;

	// TODO: Move to shared base class.
	ResultSet Query(const std::wstring &str)
	{
		return Query(str.c_str());
	}

private:

	Connection(const Connection &) = delete;
	Connection &operator=(const Connection &) = delete;

	CComPtr<IWbemLocator> m_locator;
	CComPtr<IWbemServices> m_services;

	_bstr_t m_queryLanguage;
};

}
