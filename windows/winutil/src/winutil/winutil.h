#pragma once

#include <cstdint>

#ifdef WINUTIL_EXPORTS
#define WINUTIL_LINKAGE __declspec(dllexport)
#else
#define WINUTIL_LINKAGE __declspec(dllimport)
#endif

#define WINUTIL_API __stdcall

typedef void (WINUTIL_API *WinUtilErrorSink)(const char *errorMessage, void *context);

enum class WINUTIL_MIGRATION_STATUS : uint32_t
{
	SUCCESS = 0,

	// Destination already exists
	ABORTED,

	// There's no backup
	NOTHING_TO_MIGRATE,

	FAILED,
};

extern "C"
WINUTIL_LINKAGE
WINUTIL_MIGRATION_STATUS
WINUTIL_API
WinUtil_MigrateAfterWindowsUpdate(
	WinUtilErrorSink errorSink,
	void *errorSinkContext
);
