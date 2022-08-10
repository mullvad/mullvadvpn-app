#pragma once

#include <wintun/wintun.h>
#include <libcommon/error.h>
#include "util.h"

class WintunDll
{
public:

	WintunDll() : dllHandle(nullptr)
	{
		auto wintunPath = GetProcessModulePath().replace_filename(L"wintun.dll");
		dllHandle = LoadLibraryExW(wintunPath.c_str(), nullptr, LOAD_WITH_ALTERED_SEARCH_PATH);

		if (nullptr == dllHandle)
		{
			THROW_WINDOWS_ERROR(GetLastError(), "LoadLibraryExW");
		}

		try
		{
			deleteDriver = getProcAddressOrThrow<WINTUN_DELETE_DRIVER_FUNC*>("WintunDeleteDriver");
		}
		catch (...)
		{
			FreeLibrary(dllHandle);
			throw;
		}
	}

	~WintunDll()
	{
		if (nullptr != dllHandle)
		{
			FreeLibrary(dllHandle);
		}
	}

	WINTUN_DELETE_DRIVER_FUNC *deleteDriver;

private:

	template<typename T>
	T getProcAddressOrThrow(const char *procName)
	{
		const T result = reinterpret_cast<T>(GetProcAddress(dllHandle, procName));

		if (nullptr == result)
		{
			THROW_WINDOWS_ERROR(GetLastError(), "GetProcAddress");
		}

		return result;
	}

	HMODULE dllHandle;
};
