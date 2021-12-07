#pragma once

#include <wireguard-nt/wireguard.h>
#include <libcommon/error.h>
#include "util.h"

class WireGuardNtDll
{
public:

	WireGuardNtDll() : dllHandle(nullptr)
	{
		auto path = GetProcessModulePath().replace_filename(L"mullvad-wireguard.dll");
		dllHandle = LoadLibraryExW(path.c_str(), nullptr, LOAD_WITH_ALTERED_SEARCH_PATH);

		if (nullptr == dllHandle)
		{
			THROW_WINDOWS_ERROR(GetLastError(), "LoadLibraryExW");
		}

		try
		{
			deleteDriver = getProcAddressOrThrow<WIREGUARD_DELETE_DRIVER_FUNC*>("WireGuardDeleteDriver");
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

	WIREGUARD_DELETE_DRIVER_FUNC *deleteDriver;

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
