#include <stdafx.h>
#include "update.h"
#include <libcommon/error.h>
#include <libcommon/filesystem.h>
#include <libcommon/memory.h>
#include <algorithm>
#include <fstream>
#include <filesystem>

namespace
{

// Jason found this to be a reliable marker of the signature check fix:
// https://git.zx2c4.com/wireguard-windows/tree/installer/customactions.c#n145
const char PATCH_MARKER[] = "Signature Hash";

}

namespace update
{

bool HasSetupApiSha2Fix()
{
	common::memory::ScopeDestructor destructor;

	common::fs::ScopedNativeFileSystem nativeFileSystem;

	const auto systemDir = common::fs::GetKnownFolderPath(FOLDERID_System, KF_FLAG_DEFAULT, NULL);
	const auto setupApiPath = std::filesystem::path(systemDir).append(L"setupapi.dll");

	const auto setupApiHandle = CreateFileW(
		setupApiPath.c_str(),
		GENERIC_READ,
		FILE_SHARE_READ,
		nullptr,
		OPEN_EXISTING,
		FILE_ATTRIBUTE_NORMAL,
		nullptr
	);

	if (INVALID_HANDLE_VALUE == setupApiHandle)
	{
		THROW_WINDOWS_ERROR(GetLastError(), "CreateFileW");
	}

	destructor += [=]() {
		CloseHandle(setupApiHandle);
	};

	const auto mapping = CreateFileMappingW(setupApiHandle, nullptr, PAGE_READONLY, 0, 0, nullptr);

	if (nullptr == mapping)
	{
		THROW_WINDOWS_ERROR(GetLastError(), "CreateFileMappingW");
	}

	destructor += [=]() {
		CloseHandle(mapping);
	};

	const auto bytes = MapViewOfFile(mapping, FILE_MAP_READ, 0, 0, 0);

	if (nullptr == bytes)
	{
		THROW_WINDOWS_ERROR(GetLastError(), "MapViewOfFile");
	}

	destructor += [=]() {
		UnmapViewOfFile(bytes);
	};

	MEMORY_BASIC_INFORMATION meminfo;

	if (0 == VirtualQuery(bytes, &meminfo, sizeof(meminfo)))
	{
		THROW_WINDOWS_ERROR(GetLastError(), "VirtualQuery");
	}

	constexpr auto PATCH_MARKER_SIZE = sizeof(PATCH_MARKER) - 1;

	if (meminfo.RegionSize < PATCH_MARKER_SIZE)
	{
		return false;
	}

	for (size_t i = 0; i <= meminfo.RegionSize - PATCH_MARKER_SIZE; i++)
	{
		if (0 == memcmp((void*)((char*)bytes + i), PATCH_MARKER, PATCH_MARKER_SIZE))
		{
			return true;
		}
	}

	return false;
}

}
