#include "stdafx.h"
#include "registrypaths.h"

//static
std::wstring RegistryPaths::InterfaceRoot(Protocol protocol)
{
	return
		std::wstring(L"SYSTEM\\CurrentControlSet\\Services\\")
		.append(Protocol::IPv4 == protocol ? L"Tcpip" : L"Tcpip6")
		.append(L"\\Parameters\\Interfaces");
}

//static
std::wstring RegistryPaths::InterfaceKey(const std::wstring &interfaceGuid, Protocol protocol)
{
	return
		InterfaceRoot(protocol)
		.append(L"\\")
		.append(interfaceGuid);
}
