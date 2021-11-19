#pragma once

#include <libshared/logging/logsink.h>
#include <stdint.h>

//
// WINDNS public API
//

#ifdef WINDNS_EXPORTS
#define WINDNS_LINKAGE __declspec(dllexport)
#else
#define WINDNS_LINKAGE __declspec(dllimport)
#endif

#define WINDNS_API __stdcall

///////////////////////////////////////////////////////////////////////////////
// Functions
///////////////////////////////////////////////////////////////////////////////

//
// WinDns_Initialize:
//
// Call this function once at startup, to acquire resources etc.
// The error callback is OPTIONAL.
//
extern "C"
WINDNS_LINKAGE
bool
WINDNS_API
WinDns_Initialize(
	MullvadLogSink logSink,
	void *logSinkContext
);

//
// WinDns_Deinitialize:
//
// Call this function once before unloading WINDNS or exiting the process.
//
extern "C"
WINDNS_LINKAGE
bool
WINDNS_API
WinDns_Deinitialize(
);

//
// WinDns_Set:
//
// Configure DNS servers on given adapter.
//
extern "C"
WINDNS_LINKAGE
bool
WINDNS_API
WinDns_Set(
	const NET_LUID *interfaceLuid,
	const wchar_t **ipv4Servers,
	uint32_t numIpv4Servers,
	const wchar_t **ipv6Servers,
	uint32_t numIpv6Servers
);
