#include "stdafx.h"
#include "resultset.h"
#include "windns/comlol.h"
#include "libcommon/error.h"

namespace wmi
{

ResultSet::ResultSet(CComPtr<IEnumWbemClassObject> rs) : m_resultset(rs)
{
}

bool ResultSet::advance()
{
	ULONG dummy;

	const auto status = m_resultset->Next(WBEM_INFINITE, 1, &m_result, &dummy);

	VALIDATE_COM(status, "Retrieve next object in COM resultset");

	return WBEM_S_FALSE != status;
}

_variant_t ResultSet::getResultProperty(const std::wstring &name)
{
	_variant_t val;

	auto status = m_result->Get(name.c_str(), 0, &val, nullptr, nullptr);

	VALIDATE_COM(status, "Retrieve COM property value");

	return val;
}

CComPtr<IWbemClassObject> ResultSet::getResult()
{
	return m_result;
}

}
