#include "stdafx.h"
#include <iostream>
#include <conio.h>
#include "windns/windns.h"

void WINDNS_API ErrorSink(const char *errorMessage, void *context)
{
	std::cout << "Error: " << errorMessage << std::endl;
}

void WINDNS_API ConfigSink(const void *configData, uint32_t dataLength, void *context)
{
	std::wcout << L"Updated config was delivered to WINDNS client code" << std::endl;
}

int main()
{
	std::wcout << L"Init: " << WinDns_Initialize(ErrorSink, nullptr) << std::endl;

	const wchar_t *servers[] =
	{
		L"8.8.8.8",
		L"1.1.1.1"
	};

	std::wcout << L"Set: " << WinDns_Set(servers, _countof(servers), ConfigSink, nullptr) << std::endl;

	std::wcout << L"Press a key to abort DNS monitoring + enforcing..." << std::endl;
	_getwch();

	std::wcout << L"Reset: " << WinDns_Reset() << std::endl;

	std::wcout << L"Set: " << WinDns_Set(servers, _countof(servers), ConfigSink, nullptr) << std::endl;

	std::wcout << L"Press a key to abort DNS monitoring + enforcing..." << std::endl;
	_getwch();

	std::wcout << L"Reset: " << WinDns_Reset() << std::endl;

	std::wcout << L"Deinit: " << WinDns_Deinitialize() << std::endl;

	return 0;
}