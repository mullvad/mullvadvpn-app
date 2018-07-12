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
