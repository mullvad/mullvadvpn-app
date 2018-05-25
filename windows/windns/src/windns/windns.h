#pragma once
#include <cstdint>

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

typedef void (WINDNS_API *WinDnsErrorSink)(const char *errorMessage, void *context);

//
// WinDns_Initialize:
//
// Call this function once at startup, to acquire resources etc.
//
// The OPTIONAL error callback is remembered and used to report exceptions that
// occur as a direct or indirect result of calling into WINDNS.
//
// (Recall that the monitoring provided by WINDNS is threaded.)
//
extern "C"
WINDNS_LINKAGE
bool
WINDNS_API
WinDns_Initialize(
	WinDnsErrorSink errorSink,
	void *errorContext
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
// Configure which DNS servers should be used and start enforcing these settings.
//
extern "C"
WINDNS_LINKAGE
bool
WINDNS_API
WinDns_Set(
	const wchar_t **servers,
	uint32_t numServers
);

//
// Windns_Reset:
//
// Revert server settings to what they were before calling WinDns_Set.
//
// (Also taking into account external changes to DNS settings that have occurred
// during the period of enforcing specific settings.)
//
extern "C"
WINDNS_LINKAGE
bool
WINDNS_API
WinDns_Reset(
);
