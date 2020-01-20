#include "stdafx.h"
#include "winutil.h"
#include "migration.h"
#include <libshared/logging/unwind.h>
#include <libcommon/valuemapper.h>
#include <stdexcept>

extern "C"
WINUTIL_LINKAGE
WINUTIL_MIGRATION_STATUS
WINUTIL_API
WinUtil_MigrateAfterWindowsUpdate(
	MullvadLogSink logSink,
	void *logSinkContext
)
{
	try
	{
		return common::ValueMapper::Map(migration::MigrateAfterWindowsUpdate(), {
			std::make_pair(migration::MigrationStatus::Success, WINUTIL_MIGRATION_STATUS_SUCCESS),
			std::make_pair(migration::MigrationStatus::Aborted, WINUTIL_MIGRATION_STATUS_ABORTED),
			std::make_pair(migration::MigrationStatus::NothingToMigrate, WINUTIL_MIGRATION_STATUS_NOTHING_TO_MIGRATE),
		});
	}
	catch (const std::exception &err)
	{
		shared::logging::UnwindAndLog(logSink, logSinkContext, err);
		return WINUTIL_MIGRATION_STATUS_FAILED;
	}
	catch (...)
	{
		return WINUTIL_MIGRATION_STATUS_FAILED;
	}
}
