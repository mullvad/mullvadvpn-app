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

typedef void (WINDNS_API *WinDnsErrorSink)(const char *errorMessage, const char **details, uint32_t numDetails, void *context);
typedef void (WINDNS_API *WinDnsConfigSink)(const void *configData, uint32_t dataLength, void *context);

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
// The 'configSink' will receive periodic callbacks with updated config data
// until you call WinDns_Reset.
//
// You should persist the config data in preparation for an eventual recovery.
//
extern "C"
WINDNS_LINKAGE
bool
WINDNS_API
WinDns_Set(
	const wchar_t **servers,
	uint32_t numServers,
	WinDnsConfigSink configSink,
	void *configContext
);

//
// Windns_Reset:
//
// Revert server settings to what they were before calling WinDns_Set.
//
// (Also taking into account external changes to DNS settings that have occurred
// during the period of enforcing specific settings.)
//
// It's safe to discard persisted config data once WinDns_Reset returns 'true'.
//
extern "C"
WINDNS_LINKAGE
bool
WINDNS_API
WinDns_Reset(
);

//
// WinDns_Recover:
//
// Recover adapter configurations from a previously persisted state.
//
// This is useful if the machine has been abruptly powered off and
// WINDNS did not get a chance to restore settings.
//
extern "C"
WINDNS_LINKAGE
bool
WINDNS_API
WinDns_Recover(
	const void *configData,
	uint32_t dataLength
);
