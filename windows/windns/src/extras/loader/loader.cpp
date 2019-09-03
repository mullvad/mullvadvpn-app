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

int main()
{
	common::trace::Trace::RegisterSink(new common::trace::ConsoleTraceSink);

	std::wcout << L"WinDns_Initialize: " << std::boolalpha << WinDns_Initialize(LogSink, nullptr) << std::endl;

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

	auto status = WinDns_Set(L"Wi-Fi", servers, _countof(servers), v6Servers, _countof(v6Servers));

	std::wcout << L"WinDns_Set: " << std::boolalpha << status << std::endl;

	std::wcout << L"WinDns_Deinitialize: " << std::boolalpha << WinDns_Deinitialize() << std::endl;

	return 0;
}
