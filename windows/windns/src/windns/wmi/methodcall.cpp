#include "stdafx.h"
#include "methodcall.h"
#include "windns/comhelpers.h"
#include <algorithm>

namespace wmi
{

void MethodCall::addArgument(const std::wstring &name, _variant_t value)
{
	m_arguments.emplace_back(Argument(name, value));
}

void MethodCall::addNullArgument(const std::wstring &name, CIMTYPE type)
{
	m_arguments.emplace_back(Argument(name, type));
}

_variant_t MethodCall::invoke(IConnection &connection, CComPtr<IWbemClassObject> instance, const std::wstring &methodName)
{
	std::for_each(m_arguments.begin(), m_arguments.end(), [&](const Argument &arg)
	{
		HRESULT status;

		if (arg.nullValue())
		{
			status = instance->Put(arg.name().c_str(), 0, nullptr, arg.type());
		}
		else
		{
			_variant_t &value = const_cast<variant_t &>(arg.value());

			status = instance->Put(arg.name().c_str(), 0, &value, 0);
		}

		VALIDATE_COM(status, "Register COM method argument");
	});

	_variant_t path;

	auto status = instance->Get(_bstr_t(L"__PATH"), 0, &path, nullptr, nullptr);

	VALIDATE_COM(status, "Get COM instance path");

	CComPtr<IWbemClassObject> result;

	status = connection.services()->ExecMethod(V_BSTR(&path), _bstr_t(methodName.c_str()), 0, nullptr, instance, &result, nullptr);

	VALIDATE_COM(status, "Execute COM method call");

	return ComGetProperty(result, L"ReturnValue");
}


//
// the following code is almost what is needed for static method calls
// just remove the path and the in-arg instance
// also, update first arg to ExecMethod
//
//_variant_t MethodCall::call(Connection &connection, CComPtr<IWbemClassObject> instance, const std::wstring &methodName)
//{
//	CComPtr<IWbemClassObject> cls;
//
//	auto status = connection.m_services->GetObject(_bstr_t(L"Win32_NetworkAdapterConfiguration"), 0, nullptr, &cls, nullptr);
//	VALIDATE_COM(status, "Resolve COM class");
//
//	CComPtr<IWbemClassObject> methodDefinition;
//
//	status = cls->GetMethod(methodName.c_str(), 0, &methodDefinition, nullptr);
//	VALIDATE_COM(status, "Resolve COM instance method");
//
//	CComPtr<IWbemClassObject> methodInstance;
//
//	status = methodDefinition->SpawnInstance(0, &methodInstance);
//	VALIDATE_COM(status, "Instantiate COM class for method call");
//
//	std::for_each(m_arguments.begin(), m_arguments.end(), [&](const Argument &arg)
//	{
//		_variant_t value(arg.value);
//
//		// This works for all values except NULL
//		auto hr = methodInstance->Put(arg.name.c_str(), 0, &value, 0);
//
//		VALIDATE_COM(hr, "Register COM method argument");
//	});
//
//	_variant_t path;
//
//	status = instance->Get(_bstr_t(L"__PATH"), 0, &path, nullptr, nullptr);
//	VALIDATE_COM(status, "Get COM instance path");
//
//	CComPtr<IWbemClassObject> result;
//
//	status = connection.m_services->ExecMethod(path.bstrVal, _bstr_t(methodName.c_str()), 0, nullptr, methodInstance, &result, nullptr/*?*/);
//	VALIDATE_COM(status, "Execute COM method call");
//
//	return ComGetProperty(result, L"ReturnValue");
//}
//




}
