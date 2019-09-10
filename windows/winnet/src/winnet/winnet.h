#pragma once

#include "../../shared/logsink.h"
#include <stdbool.h>

#ifdef WINNET_EXPORTS
#define WINNET_LINKAGE __declspec(dllexport)
#else
#define WINNET_LINKAGE __declspec(dllimport)
#endif

#define WINNET_API __stdcall

enum WINNET_ETM_STATUS
{
	WINNET_ETM_STATUS_METRIC_NO_CHANGE = 0,
	WINNET_ETM_STATUS_METRIC_SET = 1,
	WINNET_ETM_STATUS_FAILURE = 2,
};

extern "C"
WINNET_LINKAGE
WINNET_ETM_STATUS
WINNET_API
WinNet_EnsureTopMetric(
	const wchar_t *deviceAlias,
	MullvadLogSink logSink,
	void *logSinkContext
);

enum WINNET_GTII_STATUS
{
	WINNET_GTII_STATUS_ENABLED = 0,
	WINNET_GTII_STATUS_DISABLED = 1,
	WINNET_GTII_STATUS_FAILURE = 2,
};

extern "C"
WINNET_LINKAGE
WINNET_GTII_STATUS
WINNET_API
WinNet_GetTapInterfaceIpv6Status(
	MullvadLogSink logSink,
	void *logSinkContext
);

extern "C"
WINNET_LINKAGE
bool
WINNET_API
WinNet_GetTapInterfaceAlias(
	wchar_t **alias,
	MullvadLogSink logSink,
	void *logSinkContext
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
	MullvadLogSink logSink,
	void *logSinkContext
);

extern "C"
WINNET_LINKAGE
void
WINNET_API
WinNet_DeactivateConnectivityMonitor(
);

enum WINNET_CC_STATUS
{
	WINNET_CC_STATUS_NOT_CONNECTED = 0,
	WINNET_CC_STATUS_CONNECTED = 1,
	WINNET_CC_STATUS_CONNECTIVITY_UNKNOWN = 2,
};

extern "C"
WINNET_LINKAGE
WINNET_CC_STATUS
WINNET_API
WinNet_CheckConnectivity(
	MullvadLogSink logSink,
	void *logSinkContext
);
