#include "stdafx.h"
#include <iostream>
#include "../../winutil/winutil.h"

void WINUTIL_API ErrorSink(const char *errorMessage, void *)
{
	std::cout << "Error: " << errorMessage << std::endl;
}

int main()
{
	const auto status = WinUtil_MigrateAfterWindowsUpdate(ErrorSink, nullptr);

	switch (status)
	{
		case WINUTIL_MIGRATION_STATUS::SUCCESS:
		{
			std::wcout << L"Success" << std::endl;
			break;
		}
		case WINUTIL_MIGRATION_STATUS::ABORTED:
		{
			std::wcout << L"Aborted" << std::endl;
			break;
		}
		case WINUTIL_MIGRATION_STATUS::NOTHING_TO_MIGRATE:
		{
			std::wcout << L"Nothing to migrate" << std::endl;
			break;
		}
		case WINUTIL_MIGRATION_STATUS::FAILED:
		{
			std::wcout << L"Failed" << std::endl;
			break;
		}
	}

    return 0;
}

