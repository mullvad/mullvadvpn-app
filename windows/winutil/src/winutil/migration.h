#pragma once

namespace migration {

enum class MigrationStatus
{
	Success = 0,

	// Destination already exists
	Aborted,

	// There's no backup
	NothingToMigrate,
};

MigrationStatus MigrateAfterWindowsUpdate();

}
