#pragma once

#include "../../shared/logsink.h"
#include <stdint.h>
#include <stdbool.h>

#ifndef WINNET_STATIC
#ifdef WINNET_EXPORTS
#define WINNET_LINKAGE __declspec(dllexport)
#else
#define WINNET_LINKAGE __declspec(dllimport)
#endif
#else
#define WINNET_LINKAGE
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
	MullvadLogSink logSink,
	void *logSinkContext
);

extern "C"
WINNET_LINKAGE
void
WINNET_API
WinNet_DeactivateConnectivityMonitor(
);

enum WINNET_IP_TYPE
{
	WINNET_IP_TYPE_IPV4 = 0,
	WINNET_IP_TYPE_IPV6 = 1,
};

typedef struct tag_WINNET_IPNETWORK
{
	WINNET_IP_TYPE type;
	uint8_t bytes[16];	// Network byte order.
	uint8_t prefix;
}
WINNET_IPNETWORK;

typedef struct tag_WINNET_IP
{
	WINNET_IP_TYPE type;
	uint8_t bytes[16];	// Network byte order.
}
WINNET_IP;

typedef struct tag_WINNET_NODE
{
	const WINNET_IP *gateway;
	const wchar_t *deviceName;
}
WINNET_NODE;

typedef struct tag_WINNET_ROUTE
{
	WINNET_IPNETWORK network;
	const WINNET_NODE *node;
}
WINNET_ROUTE;

extern "C"
WINNET_LINKAGE
bool
WINNET_API
WinNet_ActivateRouteManager(
	MullvadLogSink logSink,
	void *logSinkContext
);

extern "C"
WINNET_LINKAGE
bool
WINNET_API
WinNet_AddRoutes(
	const WINNET_ROUTE *routes,
	uint32_t numRoutes
);

extern "C"
WINNET_LINKAGE
bool
WINNET_API
WinNet_AddRoute(
	const WINNET_ROUTE *route
);

extern "C"
WINNET_LINKAGE
bool
WINNET_API
WinNet_DeleteRoutes(
	const WINNET_ROUTE *routes,
	uint32_t numRoutes
);

extern "C"
WINNET_LINKAGE
bool
WINNET_API
WinNet_DeleteRoute(
	const WINNET_ROUTE *route
);

enum WINNET_DEFAULT_ROUTE_CHANGED_EVENT_TYPE
{
	// Best default route changed.
	WINNET_DEFAULT_ROUTE_CHANGED_EVENT_TYPE_UPDATED = 0,

	// No default routes exist.
	WINNET_DEFAULT_ROUTE_CHANGED_EVENT_TYPE_REMOVED = 1,
};

enum WINNET_IP_FAMILY
{
	WINNET_IP_FAMILY_V4 = 0,
	WINNET_IP_FAMILY_V6 = 1,
};

typedef void (WINNET_API *WinNetDefaultRouteChangedCallback)
(
	WINNET_DEFAULT_ROUTE_CHANGED_EVENT_TYPE eventType,

	// Signals which IP family the event relates to.
	WINNET_IP_FAMILY family,

	// For update events, signals the interface associated with the new best default route.
	uint64_t interfaceLuid,

	void *context
);

extern "C"
WINNET_LINKAGE
bool
WINNET_API
WinNet_RegisterDefaultRouteChangedCallback(
	WinNetDefaultRouteChangedCallback callback,
	void *context,
	void **registrationHandle
);

extern "C"
WINNET_LINKAGE
void
WINNET_API
WinNet_UnregisterDefaultRouteChangedCallback(
	void *registrationHandle
);

extern "C"
WINNET_LINKAGE
void
WINNET_API
WinNet_DeactivateRouteManager(
);

extern "C"
WINNET_LINKAGE
bool
WINNET_API
WinNet_AddDeviceIpAddresses(
	const wchar_t *deviceAlias,
	const WINNET_IP *addresses,
	uint32_t numAddresses,
	MullvadLogSink logSink,
	void *logSinkContext
);

