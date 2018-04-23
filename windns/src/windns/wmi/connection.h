#pragma once

#include "iconnection.h"
#include <string>
#define _WIN32_DCOM
#include <windows.h>
#include <atlbase.h>
#include <comutil.h>
#include <comdef.h>

#pragma comment(lib, "wbemuuid.lib")

namespace wmi
{

class Connection : public IConnection
{
public:

	enum class Namespace
	{
		Default,
		Cimv2,
		StandardCimv2
	};

	explicit Connection(Namespace ns);

	ResultSet query(const wchar_t *query) override;

	CComPtr<IWbemServices> services() override
	{
		return m_services;
	}

private:

	CComPtr<IWbemLocator> m_locator;
	CComPtr<IWbemServices> m_services;

	_bstr_t m_queryLanguage;
};

}
