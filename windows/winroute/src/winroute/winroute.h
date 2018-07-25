#pragma once
#include <cstdint>

#ifdef WINROUTE_EXPORTS
#define WINROUTE_LINKAGE __declspec(dllexport)
#else
#define WINROUTE_LINKAGE __declspec(dllimport)
#endif

#define WINROUTE_API __stdcall

typedef void (WINROUTE_API *WinRouteErrorSink)(const char *errorMessage, void *context);

enum class WINROUTE_STATUS : uint32_t
{
	METRIC_NO_CHANGE = 0,
	METRIC_SET = 1,
	FAILURE = 2,
};


extern "C"
WINROUTE_LINKAGE
WINROUTE_STATUS
WINROUTE_API
WinRoute_EnsureTopMetric(
	const wchar_t *deviceAlias,
	WinRouteErrorSink errorSink,
	void* errorSinkContext
);

enum class TAP_IPV6_STATUS : uint32_t
{
	ENABLED = 0,
	DISABLED = 1,
	FAILURE = 2,
};

//
// This has nothing to do with routing.
// We should probably rename this module and use it to gather one-off network functions.
//
extern "C"
WINROUTE_LINKAGE
TAP_IPV6_STATUS
WINROUTE_API
GetTapInterfaceIpv6Status(
	WinRouteErrorSink errorSink,
	void* errorSinkContext
);
