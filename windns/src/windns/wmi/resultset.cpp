#include "stdafx.h"
#include "resultset.h"
#include "windns/comhelpers.h"
#include "libcommon/error.h"

namespace wmi
{

ResultSet::ResultSet(CComPtr<IEnumWbemClassObject> rs) : m_resultset(rs)
{
}

bool ResultSet::advance()
{
	if (nullptr != m_result)
	{
		m_result.Release();
	}

	ULONG dummy;

	const auto status = m_resultset->Next(WBEM_INFINITE, 1, &m_result, &dummy);

	VALIDATE_COM(status, "Retrieve next object in COM resultset");

	return WBEM_S_FALSE != status;
}

CComPtr<IWbemClassObject> ResultSet::result()
{
	return m_result;
}

}
