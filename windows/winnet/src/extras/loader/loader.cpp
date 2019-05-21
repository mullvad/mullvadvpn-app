#include "stdafx.h"
#include "../../winnet/winnet.h"
#include <iostream>

void __stdcall ConnectivityChanged(uint8_t connected)
{
	std::wcout << (0 != connected? L"Connected" : L"NOT connected") << std::endl;
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

	uint8_t currentConnectivity = 0;

	const auto status = WinNet_ActivateConnectivityMonitor(ConnectivityChanged, &currentConnectivity, nullptr, nullptr);

	std::wcout << L"Current connectivity: "
		<< (0 != currentConnectivity ? L"Connected" : L"NOT connected") << std::endl;

	_getwch();

    return 0;
}

