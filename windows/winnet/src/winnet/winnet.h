#pragma once
#include <cstdint>

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
	void* errorSinkContext
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
	void* errorSinkContext
);

enum class WINNET_GTIA_STATUS : uint32_t
{
	SUCCESS = 0,
	FAILURE = 1,
};

extern "C"
WINNET_LINKAGE
WINNET_GTIA_STATUS
WINNET_API
WinNet_GetTapInterfaceAlias(
	wchar_t **alias,
	WinNetErrorSink errorSink,
	void* errorSinkContext
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

typedef void (WINNET_API *WinNetConnectivityMonitorCallback)(uint8_t connected);

enum class WINNET_ACM_STATUS : uint32_t
{
	SUCCESS = 0,
	FAILURE = 1,
};

extern "C"
WINNET_LINKAGE
WINNET_ACM_STATUS
WINNET_API
WinNet_ActivateConnectivityMonitor(
	WinNetConnectivityMonitorCallback callback,
	uint8_t *currentConnectivity,
	WinNetErrorSink errorSink,
	void* errorSinkContext
);

extern "C"
WINNET_LINKAGE
void
WINNET_API
WinNet_DeactivateConnectivityMonitor(
);
