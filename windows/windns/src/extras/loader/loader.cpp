#include "stdafx.h"
#include "windns/windns.h"
#include <iostream>
#include <conio.h>
#include <vector>
#include <windows.h>

void WINDNS_API ErrorSink(const char *errorMessage, void *context)
{
	std::cout << "WINDNS Error: " << errorMessage << std::endl;
}

void WINDNS_API ConfigSink(const void *configData, uint32_t dataLength, void *context)
{
	std::wcout << L"Updated config was delivered to WINDNS client code" << std::endl;

	auto f = CreateFileW(L"windns_recovery", GENERIC_WRITE, 0, nullptr, CREATE_ALWAYS, 0, nullptr);

	if (INVALID_HANDLE_VALUE == f)
	{
		std::wcout << L"Failed to create recovery file" << std::endl;
		return;
	}

	if (FALSE == WriteFile(f, configData, dataLength, nullptr, nullptr))
	{
		std::wcout << L"Failed to update recovery file" << std::endl;
	}

	CloseHandle(f);
}

void Recover()
{
	auto f = CreateFileW(L"windns_recovery", GENERIC_READ, FILE_SHARE_READ, nullptr, OPEN_EXISTING, 0, nullptr);

	if (INVALID_HANDLE_VALUE == f)
	{
		std::wcout << L"Failed to open recovery file" << std::endl;
		return;
	}

	std::vector<uint8_t> data;

	data.resize(GetFileSize(f, nullptr));

	if (FALSE == ReadFile(f, &data[0], static_cast<DWORD>(data.size()), nullptr, nullptr))
	{
		std::wcout << L"Failed to read in recovery data" << std::endl;
		CloseHandle(f);

		return;
	}

	std::wcout << L"WinDns_Recover: " << WinDns_Recover(&data[0], static_cast<uint32_t>(data.size())) << std::endl;
}

bool Ask(const std::wstring &question)
{
	std::wcout << question.c_str() << L" Y/N: ";

	auto answer = _getwch();

	std::wcout << std::endl;

	if ('y' == answer || 'Y' == answer)
	{
		return true;
	}

	return false;
}

void WaitInput(const std::wstring &message)
{
	std::wcout << message.c_str() << std::endl;
	_getwch();
}

int main()
{
	if (Ask(L"Perform recovery?"))
	{
		Recover();
		return 0;
	}

	std::wcout << L"WinDns_Initialize: " << WinDns_Initialize(ErrorSink, nullptr) << std::endl;

	const wchar_t *servers[] =
	{
		L"8.8.8.8"
	};

	std::wcout << L"WinDns_Set: " << WinDns_Set(servers, _countof(servers), ConfigSink, nullptr) << std::endl;

	WaitInput(L"Press a key to abort DNS monitoring + enforcing...");

	if (Ask(L"Perform WinDns_Reset() before next WinDns_Set()?"))
	{
		std::wcout << L"WinDns_Reset: " << WinDns_Reset() << std::endl;
	}

	std::wcout << L"WinDns_Set: " << WinDns_Set(servers, _countof(servers), ConfigSink, nullptr) << std::endl;

	WaitInput(L"Press a key to abort DNS monitoring + enforcing...");

	if (Ask(L"Perform WinDns_Reset() before WinDns_Deinitialize()?"))
	{
		std::wcout << L"WinDns_Reset: " << WinDns_Reset() << std::endl;
	}

	std::wcout << L"WinDns_Deinitialize: " << WinDns_Deinitialize() << std::endl;

	return 0;
}