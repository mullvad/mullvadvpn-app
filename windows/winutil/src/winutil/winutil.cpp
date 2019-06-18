#include "stdafx.h"
#include "winutil.h"
#include "migration.h"
#include "libcommon/valuemapper.h"
#include <stdexcept>

extern "C"
WINUTIL_LINKAGE
WINUTIL_MIGRATION_STATUS
WINUTIL_API
WinUtil_MigrateAfterWindowsUpdate(
	WinUtilErrorSink errorSink,
	void *errorSinkContext
)
{
	try
	{
		using value_type = common::ValueMapper<migration::MigrationStatus, WINUTIL_MIGRATION_STATUS>::value_type;

		const common::ValueMapper<migration::MigrationStatus, WINUTIL_MIGRATION_STATUS> mapper =
		{
			value_type(migration::MigrationStatus::Success, WINUTIL_MIGRATION_STATUS::SUCCESS),
			value_type(migration::MigrationStatus::Aborted, WINUTIL_MIGRATION_STATUS::ABORTED),
			value_type(migration::MigrationStatus::NothingToMigrate, WINUTIL_MIGRATION_STATUS::NOTHING_TO_MIGRATE),
		};

		return mapper.map(migration::MigrateAfterWindowsUpdate());
	}
	catch (const std::exception &err)
	{
		if (nullptr != errorSink)
		{
			errorSink(err.what(), errorSinkContext);
		}

		return WINUTIL_MIGRATION_STATUS::FAILED;
	}
	catch (...)
	{
		return WINUTIL_MIGRATION_STATUS::FAILED;
	}
}
