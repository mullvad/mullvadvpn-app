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
			createAdapter = getProcAddressOrThrow<WINTUN_CREATE_ADAPTER_FUNC>("WintunCreateAdapter");
			openAdapter = getProcAddressOrThrow<WINTUN_OPEN_ADAPTER_FUNC>("WintunOpenAdapter");
			freeAdapter = getProcAddressOrThrow<WINTUN_FREE_ADAPTER_FUNC>("WintunFreeAdapter");
			deletePoolDriver = getProcAddressOrThrow<WINTUN_DELETE_POOL_DRIVER_FUNC>("WintunDeletePoolDriver");
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

	WINTUN_CREATE_ADAPTER_FUNC createAdapter;
	WINTUN_OPEN_ADAPTER_FUNC openAdapter;
	WINTUN_FREE_ADAPTER_FUNC freeAdapter;
	WINTUN_DELETE_POOL_DRIVER_FUNC deletePoolDriver;

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
