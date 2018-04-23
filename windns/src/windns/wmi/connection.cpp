#include "stdafx.h"
#include "connection.h"
#include "windns/comhelpers.h"
#include <stdexcept>
#define _WIN32_DCOM
#include <windows.h>
#include <wbemidl.h>

namespace
{

const wchar_t *LiteralNamespace(wmi::Connection::Namespace ns)
{
	switch (ns)
	{
		case wmi::Connection::Namespace::Default: return L"root\\Default";
		case wmi::Connection::Namespace::Cimv2: return L"root\\CIMV2";
		case wmi::Connection::Namespace::StandardCimv2: return L"root\\StandardCIMV2";
		default:
		{
			throw std::logic_error("Missing case handler in switch clause");
		}
	}
}

} // anonymous namespace

namespace wmi
{

Connection::Connection(Namespace ns) : m_queryLanguage(L"WQL")
{
	auto status = CoCreateInstance(CLSID_WbemLocator, nullptr, CLSCTX_INPROC_SERVER,
		IID_IWbemLocator, (LPVOID *)&m_locator);

	if (CO_E_NOTINITIALIZED == status)
	{
		VALIDATE_COM(CoInitializeEx(nullptr, COINIT_MULTITHREADED), "Initialize COM");

		status = CoCreateInstance(CLSID_WbemLocator, nullptr, CLSCTX_INPROC_SERVER,
			IID_IWbemLocator, (LPVOID *)&m_locator);
	}

	VALIDATE_COM(status, "Create COM locator instance");

	status = m_locator->ConnectServer(_bstr_t(LiteralNamespace(ns)), nullptr, nullptr,
		nullptr, 0, nullptr, nullptr, &m_services);

	VALIDATE_COM(status, "Create COM services instance");

	status = CoSetProxyBlanket(m_services, RPC_C_AUTHN_WINNT, RPC_C_AUTHZ_NONE, nullptr,
		RPC_C_AUTHN_LEVEL_CALL, RPC_C_IMP_LEVEL_IMPERSONATE, nullptr, EOAC_NONE);

	VALIDATE_COM(status, "Configure COM services auth");
}

ResultSet Connection::query(const wchar_t *query)
{
	CComPtr<IEnumWbemClassObject> result;

	auto status = m_services->ExecQuery(m_queryLanguage, _bstr_t(query),
		WBEM_FLAG_FORWARD_ONLY | WBEM_FLAG_RETURN_IMMEDIATELY, nullptr, &result);

	VALIDATE_COM(status, "Execute WMI query");

	return ResultSet(result);
}

}
