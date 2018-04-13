#pragma once

#include "resultset.h"

namespace wmi
{

struct IConnection
{
	virtual ~IConnection() = 0
	{
	}

	virtual ResultSet Query(const wchar_t *query) = 0;
};

}
