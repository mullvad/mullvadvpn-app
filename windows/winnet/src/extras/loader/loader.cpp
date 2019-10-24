#include "stdafx.h"
#include "../../winnet/winnet.h"
#include <iostream>

void __stdcall ConnectivityChanged(bool connected, void *)
{
	std::wcout << (0 != connected? L"Connected" : L"NOT connected") << std::endl;
}

namespace
{

void __stdcall log(MULLVAD_LOG_SINK_SEVERITY severity, const char *msg, void *)
{
	switch (severity)
	{
	case MULLVAD_LOG_SINK_SEVERITY_ERROR:
		std::cout << "Error: ";
		break;
	case MULLVAD_LOG_SINK_SEVERITY_WARNING:
		std::cout << "Warning: ";
		break;
	case MULLVAD_LOG_SINK_SEVERITY_INFO:
		std::cout << "Info: ";
		break;
	case MULLVAD_LOG_SINK_SEVERITY_TRACE:
		std::cout << "Trace: ";
		break;
	}

	std::cout << msg << std::endl;
}

}

int main()
{
	//wchar_t *alias = nullptr;

	//const auto status = WinNet_GetTapInterfaceAlias(&alias, nullptr, nullptr);

	//switch (status)
	//{
	//	case WINNET_GTIA_STATUS::FAILURE:
	//	{
	//		std::wcout << L"Could not determine alias" << std::endl;
	//		break;
	//	}
	//	case WINNET_GTIA_STATUS::SUCCESS:
	//	{
	//		std::wcout << L"Interface alias: " << alias << std::endl;
	//		WinNet_ReleaseString(alias);
	//	}
	//};

	const auto status = WinNet_ActivateConnectivityMonitor(
		ConnectivityChanged,
		nullptr,
		log,
		nullptr
	);

	_getwch();

    return 0;
}

