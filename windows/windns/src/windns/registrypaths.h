#pragma once

#include "types.h"
#include <string>

class RegistryPaths
{
public:

	RegistryPaths() = delete;

	static std::wstring InterfaceRoot(Protocol protocol);
	static std::wstring InterfaceKey(const std::wstring &interfaceGuid, Protocol protocol);
};
