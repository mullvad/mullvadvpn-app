#include "stdafx.h"
#include "windns/windns.h"
#include <libshared/logging/stdoutlogger.h>
#include <libcommon/trace/trace.h>
#include <libcommon/trace/consoletracesink.h>
#include <iostream>
#include <conio.h>
#include <vector>
#include <windows.h>

int main()
{
	common::trace::Trace::RegisterSink(new common::trace::ConsoleTraceSink);

	std::wcout << L"WinDns_Initialize: " << std::boolalpha << WinDns_Initialize(shared::logging::StdoutLogger, nullptr) << std::endl;

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
