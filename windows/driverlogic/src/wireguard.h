#pragma once

#include <wireguard-nt/wireguard.h>
#include <libcommon/error.h>
#include "util.h"

class WireGuardNtDll
{
public:

	WireGuardNtDll() : dllHandle(nullptr)
	{
		auto path = GetProcessModulePath().replace_filename(L"wireguard.dll");
		dllHandle = LoadLibraryExW(path.c_str(), nullptr, LOAD_WITH_ALTERED_SEARCH_PATH);

		if (nullptr == dllHandle)
		{
			THROW_WINDOWS_ERROR(GetLastError(), "LoadLibraryExW");
		}

		try
		{
			deletePoolDriver = getProcAddressOrThrow<WIREGUARD_DELETE_POOL_DRIVER_FUNC*>("WireGuardDeletePoolDriver");
		}
		catch (...)
		{
			FreeLibrary(dllHandle);
			throw;
		}
	}

	~WireGuardNtDll()
	{
		if (nullptr != dllHandle)
		{
			FreeLibrary(dllHandle);
		}
	}

	WIREGUARD_DELETE_POOL_DRIVER_FUNC *deletePoolDriver;

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
