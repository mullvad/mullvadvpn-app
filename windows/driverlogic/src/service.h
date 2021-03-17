#pragma once

#include <windows.h>

void WaitUntilServiceStopped(SC_HANDLE service, DWORD maxWaitMs);

void PokeService(const std::wstring &serviceName, bool stopService, bool deleteService);
