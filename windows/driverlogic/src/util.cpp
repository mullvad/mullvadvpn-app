#include "stdafx.h"
#include "util.h"
#include <windows.h>
#include <vector>
#include <libcommon/error.h>

using path = std::filesystem::path;

path
GetProcessModulePath
(
)
{
	size_t bufferSize = MAX_PATH;

	std::vector<wchar_t> pathBuffer(bufferSize);

	for (;;)
	{
		const auto writtenChars = GetModuleFileNameW(nullptr, &pathBuffer[0], static_cast<DWORD>(pathBuffer.size()));

		if (0 == writtenChars)
		{
			THROW_WINDOWS_ERROR(GetLastError(), "GetModuleFileNameW");
		}

		if (writtenChars != pathBuffer.size())
		{
			return path(pathBuffer.begin(), pathBuffer.begin() + writtenChars);
		}

		bufferSize *= 2;

		pathBuffer.resize(bufferSize);
	}
}
