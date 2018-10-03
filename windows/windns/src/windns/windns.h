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

enum WinDnsLogCategory
{
	WINDNS_LOG_CATEGORY_ERROR	= 0x01,
	WINDNS_LOG_CATEGORY_INFO	= 0x02
};

typedef void (WINDNS_API *WinDnsLogSink)(WinDnsLogCategory category, const char *message,
	const char **details, uint32_t numDetails, void *context);

typedef void (WINDNS_API *WinDnsRecoverySink)(const void *recoveryData, uint32_t dataLength, void *context);

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
	WinDnsLogSink logSink,
	void *logContext
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
// The 'recoverySink' will receive periodic callbacks with updated recovery data
// until you call WinDns_Reset.
//
// You should persist the recovery data in preparation for an eventual recovery.
//
extern "C"
WINDNS_LINKAGE
bool
WINDNS_API
WinDns_Set(
	const wchar_t **ipv4Servers,
	uint32_t numIpv4Servers,
	const wchar_t **ipv6Servers,
	uint32_t numIpv6Servers,
	WinDnsRecoverySink recoverySink,
	void *recoveryContext
);

//
// Windns_Reset:
//
// Revert server settings to what they were before calling WinDns_Set.
//
// (Also taking into account external changes to DNS settings that have occurred
// during the period of enforcing specific settings.)
//
// It's safe to discard persisted recovery data once WinDns_Reset returns 'true'.
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
	const void *recoveryData,
	uint32_t dataLength
);
