#include <stdafx.h>
#include "update.h"
#include <libcommon/error.h>
#include <libcommon/filesystem.h>
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
	common::fs::ScopedNativeFileSystem nativeFileSystem;

	const auto systemDir = common::fs::GetKnownFolderPath(FOLDERID_System, KF_FLAG_DEFAULT, NULL);
	const auto setupApiPath = std::filesystem::path(systemDir).append(L"setupapi.dll");

	std::ifstream ifs(setupApiPath, std::ios_base::binary);
	if (!ifs)
	{
		// Maybe sketchy to rely on GLE here
		THROW_WINDOWS_ERROR(GetLastError(), "Failed to open setupapi.dll");
	}

	const auto marker_end = PATCH_MARKER + sizeof(PATCH_MARKER) - sizeof('\0');
	const auto last = std::istreambuf_iterator<char>();
	return (last != std::search(std::istreambuf_iterator<char>(ifs), last, PATCH_MARKER, marker_end));
}

}
