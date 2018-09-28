#include "stdafx.h"
#include "windns/windns.h"
#include "libcommon/trace/trace.h"
#include "libcommon/trace/consoletracesink.h"
#include <iostream>
#include <conio.h>
#include <vector>
#include <windows.h>

void WINDNS_API LogSink(WinDnsLogCategory category, const char *message, const char **details,
	uint32_t numDetails, void *context)
{
	if (WINDNS_LOG_CATEGORY_ERROR == category)
	{
		std::cout << "WINDNS Error: ";
	}
	else
	{
		std::cout << "WINDNS Info: ";
	}

	std::cout << message << std::endl;

	for (uint32_t i = 0; i < numDetails; ++i)
	{
		std::cout << "    " << details[i] << std::endl;
	}
}

void WINDNS_API RecoverySink(const void *recoveryData, uint32_t dataLength, void *context)
{
	std::wcout << L"Updated recovery data was delivered to WINDNS client code" << std::endl;

	auto f = CreateFileW(L"windns_recovery", GENERIC_WRITE, 0, nullptr, CREATE_ALWAYS, 0, nullptr);

	if (INVALID_HANDLE_VALUE == f)
	{
		std::wcout << L"Failed to create recovery file" << std::endl;
		return;
	}

	if (FALSE == WriteFile(f, recoveryData, dataLength, nullptr, nullptr))
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

	std::wcout << L"WinDns_Recover: " << std::boolalpha <<
		WinDns_Recover(&data[0], static_cast<uint32_t>(data.size())) << std::endl;
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
	common::trace::Trace::RegisterSink(new common::trace::ConsoleTraceSink);

	std::wcout << L"WinDns_Initialize: " << std::boolalpha << WinDns_Initialize(LogSink, nullptr) << std::endl;

	if (Ask(L"Perform recovery?"))
	{
		Recover();
		return 0;
	}

	const wchar_t *servers[] =
	{
		L"8.8.8.8",
		L"8.8.4.4"
	};

	const wchar_t *v6Servers[] =
	{
		L"2001:4860:4860::8888",
		L"2001:4860:4860::8844"
	};

	auto status = WinDns_Set(servers, _countof(servers), v6Servers, _countof(v6Servers), RecoverySink, nullptr);

	std::wcout << L"WinDns_Set: " << std::boolalpha << status << std::endl;

	WaitInput(L"Press a key to abort DNS monitoring + enforcing...");

	if (Ask(L"Perform WinDns_Reset() before next WinDns_Set()?"))
	{
		std::wcout << L"WinDns_Reset: " << std::boolalpha << WinDns_Reset() << std::endl;
	}

	status = WinDns_Set(servers, _countof(servers), v6Servers, _countof(v6Servers), RecoverySink, nullptr);

	std::wcout << L"WinDns_Set: " << std::boolalpha << status << std::endl;

	WaitInput(L"Press a key to abort DNS monitoring + enforcing...");

	if (Ask(L"Perform WinDns_Reset() before WinDns_Deinitialize()?"))
	{
		std::wcout << L"WinDns_Reset: " << std::boolalpha << WinDns_Reset() << std::endl;
	}

	std::wcout << L"WinDns_Deinitialize: " << std::boolalpha << WinDns_Deinitialize() << std::endl;

	return 0;
}
