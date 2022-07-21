#pragma once

#include <windows.h>

bool ServiceIsRunning(const std::wstring &serviceName);

void PokeService(const std::wstring &serviceName, bool stopService, bool deleteService);
