#pragma once

#include <libshared/logging/logsink.h>
#include <stdint.h>

#ifdef WINUTIL_EXPORTS
#define WINUTIL_LINKAGE __declspec(dllexport)
#else
#define WINUTIL_LINKAGE __declspec(dllimport)
#endif

#define WINUTIL_API __stdcall

enum WINUTIL_MIGRATION_STATUS
{
	WINUTIL_MIGRATION_STATUS_SUCCESS = 0,

	// Destination already exists
	WINUTIL_MIGRATION_STATUS_ABORTED,

	// There's no backup
	WINUTIL_MIGRATION_STATUS_NOTHING_TO_MIGRATE,

	WINUTIL_MIGRATION_STATUS_FAILED,
};

extern "C"
WINUTIL_LINKAGE
WINUTIL_MIGRATION_STATUS
WINUTIL_API
WinUtil_MigrateAfterWindowsUpdate(
	MullvadLogSink logSink,
	void *logSinkContext
);
