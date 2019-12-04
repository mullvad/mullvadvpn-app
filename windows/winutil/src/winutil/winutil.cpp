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
		using value_type = common::ValueMapper<migration::MigrationStatus, WINUTIL_MIGRATION_STATUS>::value_type;

		const common::ValueMapper<migration::MigrationStatus, WINUTIL_MIGRATION_STATUS> mapper =
		{
			value_type(migration::MigrationStatus::Success, WINUTIL_MIGRATION_STATUS_SUCCESS),
			value_type(migration::MigrationStatus::Aborted, WINUTIL_MIGRATION_STATUS_ABORTED),
			value_type(migration::MigrationStatus::NothingToMigrate, WINUTIL_MIGRATION_STATUS_NOTHING_TO_MIGRATE),
		};

		return mapper.map(migration::MigrateAfterWindowsUpdate());
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
