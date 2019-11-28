#include "stdafx.h"
#include <iostream>
#include <libshared/logging/stdoutlogger.h>
#include <winutil/winutil.h>

int main()
{
	const auto status = WinUtil_MigrateAfterWindowsUpdate(shared::logging::StdoutLogger, nullptr);

	switch (status)
	{
		case WINUTIL_MIGRATION_STATUS_SUCCESS:
		{
			std::wcout << L"Success" << std::endl;
			break;
		}
		case WINUTIL_MIGRATION_STATUS_ABORTED:
		{
			std::wcout << L"Aborted" << std::endl;
			break;
		}
		case WINUTIL_MIGRATION_STATUS_NOTHING_TO_MIGRATE:
		{
			std::wcout << L"Nothing to migrate" << std::endl;
			break;
		}
		case WINUTIL_MIGRATION_STATUS_FAILED:
		{
			std::wcout << L"Failed" << std::endl;
			break;
		}
	}

    return 0;
}
