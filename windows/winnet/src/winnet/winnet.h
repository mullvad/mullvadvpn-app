#pragma once

#include <libshared/logging/logsink.h>
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

enum WINNET_EBM_STATUS
{
	WINNET_EBM_STATUS_METRIC_NO_CHANGE = 0,
	WINNET_EBM_STATUS_METRIC_SET = 1,
	WINNET_EBM_STATUS_FAILURE = 2,
};

extern "C"
WINNET_LINKAGE
WINNET_EBM_STATUS
WINNET_API
WinNet_EnsureBestMetric(
	const wchar_t *deviceAlias,
	MullvadLogSink logSink,
	void *logSinkContext
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

enum WINNET_ADDR_FAMILY
{
	WINNET_ADDR_FAMILY_IPV4 = 0,
	WINNET_ADDR_FAMILY_IPV6 = 1,
};

typedef struct tag_WINNET_IP
{
	WINNET_ADDR_FAMILY family;
	uint8_t bytes[16];	// Network byte order.
}
WINNET_IP;

typedef struct tag_WINNET_IP_NETWORK
{
	uint8_t prefix;
	WINNET_IP addr;
}
WINNET_IP_NETWORK;

typedef struct tag_WINNET_NODE
{
	const WINNET_IP *gateway;
	const wchar_t *deviceName;
}
WINNET_NODE;

typedef struct tag_WINNET_ROUTE
{
	WINNET_IP_NETWORK network;
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

enum WINNET_AR_STATUS
{
	WINNET_AR_STATUS_SUCCESS = 0,
	WINNET_AR_STATUS_GENERAL_ERROR = 1,
	WINNET_AR_STATUS_NO_DEFAULT_ROUTE = 2,
	WINNET_AR_STATUS_NAME_NOT_FOUND = 3,
	WINNET_AR_STATUS_GATEWAY_NOT_FOUND = 4,
};

extern "C"
WINNET_LINKAGE
WINNET_AR_STATUS
WINNET_API
WinNet_AddRoutes(
	const WINNET_ROUTE *routes,
	uint32_t numRoutes
);

extern "C"
WINNET_LINKAGE
WINNET_AR_STATUS
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

extern "C"
WINNET_LINKAGE
bool
WINNET_API
WinNet_DeleteAppliedRoutes(
);

typedef struct tag_WINNET_DEFAULT_ROUTE
{
	uint64_t interfaceLuid;
	WINNET_IP gateway;
}
WINNET_DEFAULT_ROUTE;

enum WINNET_STATUS
{
	WINNET_STATUS_SUCCESS = 0,
	WINNET_STATUS_NOT_FOUND = 1,
	WINNET_STATUS_FAILURE = 2,
};

extern "C"
WINNET_LINKAGE
WINNET_STATUS
WINNET_API
WinNet_GetBestDefaultRoute(
	WINNET_ADDR_FAMILY family,
	WINNET_DEFAULT_ROUTE *route,
	MullvadLogSink logSink,
	void *logSinkContext
);

extern "C"
WINNET_LINKAGE
WINNET_STATUS
WINNET_API
WinNet_InterfaceLuidToIpAddress(
	WINNET_ADDR_FAMILY family,
	uint64_t interfaceLuid,
	WINNET_IP *ip,
	MullvadLogSink logSink,
	void *logSinkContext
);

enum WINNET_DEFAULT_ROUTE_CHANGED_EVENT_TYPE
{
	// Best default route changed.
	WINNET_DEFAULT_ROUTE_CHANGED_EVENT_TYPE_UPDATED = 0,

	// No default routes exist.
	WINNET_DEFAULT_ROUTE_CHANGED_EVENT_TYPE_REMOVED = 1,
};

typedef void (WINNET_API *WinNetDefaultRouteChangedCallback)
(
	WINNET_DEFAULT_ROUTE_CHANGED_EVENT_TYPE eventType,

	// Indicates which IP family the event relates to.
	WINNET_ADDR_FAMILY family,

	// For update events, indicates the interface associated with the new best default route.
	WINNET_DEFAULT_ROUTE route,

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

