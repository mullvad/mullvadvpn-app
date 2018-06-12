#pragma once
#include <cstdint>

#ifdef WINROUTE_EXPORTS
#define WINROUTE_LINKAGE __declspec(dllexport)
#else
#define WINROUTE_LINKAGE __declspec(dllimport)
#endif

#define WINROUTE_API __stdcall

typedef void (WINROUTE_API *WinRouteErrorSink)(const char *errorMessage, void *context);


extern "C"
WINROUTE_LINKAGE
int32_t
WINROUTE_API
WinRoute_EnsureTopMetric(
		const wchar_t *deviceAlias,
		WinRouteErrorSink errorSink,
	    void* errorSinkContext
);