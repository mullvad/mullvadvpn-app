#pragma once

#include "resultset.h"
#include <atlbase.h>
#include <wbemidl.h>

namespace wmi
{

struct IConnection
{
	virtual ~IConnection() = 0
	{
	}

	virtual ResultSet query(const wchar_t *query) = 0;
	virtual CComPtr<IWbemServices> services() = 0;
};

}
