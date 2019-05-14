#pragma once
#include <stdint.h>
#include <stdbool.h>

#ifdef WINNET_EXPORTS
#define WINNET_LINKAGE __declspec(dllexport)
#else
#define WINNET_LINKAGE __declspec(dllimport)
#endif

#define WINNET_API __stdcall

typedef void (WINNET_API *WinNetErrorSink)(const char *errorMessage, void *context);

enum class WINNET_ETM_STATUS : uint32_t
{
	METRIC_NO_CHANGE = 0,
	METRIC_SET = 1,
	FAILURE = 2,
};

extern "C"
WINNET_LINKAGE
WINNET_ETM_STATUS
WINNET_API
WinNet_EnsureTopMetric(
	const wchar_t *deviceAlias,
	WinNetErrorSink errorSink,
	void *errorSinkContext
);

enum class WINNET_GTII_STATUS : uint32_t
{
	ENABLED = 0,
	DISABLED = 1,
	FAILURE = 2,
};

extern "C"
WINNET_LINKAGE
WINNET_GTII_STATUS
WINNET_API
WinNet_GetTapInterfaceIpv6Status(
	WinNetErrorSink errorSink,
	void *errorSinkContext
);

extern "C"
WINNET_LINKAGE
bool
WINNET_API
WinNet_GetTapInterfaceAlias(
	wchar_t **alias,
	WinNetErrorSink errorSink,
	void *errorSinkContext
);

//
// This is a companion function to the above function.
// Generically named in case we need other functions here that return strings.
//
extern "C"
WINNET_LINKAGE
void
WINNET_API
WinNet_ReleaseString(
	wchar_t *str
);

typedef void (WINNET_API *WinNetConnectivityMonitorCallback)(bool connected, void *context);

extern "C"
WINNET_LINKAGE
bool
WINNET_API
WinNet_ActivateConnectivityMonitor(
	WinNetConnectivityMonitorCallback callback,
	void *callbackContext,
	bool *currentConnectivity,
	WinNetErrorSink errorSink,
	void *errorSinkContext
);

extern "C"
WINNET_LINKAGE
void
WINNET_API
WinNet_DeactivateConnectivityMonitor(
);

enum class WINNET_CC_STATUS : uint32_t
{
	NOT_CONNECTED = 0,
	CONNECTED = 1,
	CONNECTIVITY_UNKNOWN = 2,
};

extern "C"
WINNET_LINKAGE
WINNET_CC_STATUS
WINNET_API
WinNet_CheckConnectivity(
	WinNetErrorSink errorSink,
	void *errorSinkContext
);
