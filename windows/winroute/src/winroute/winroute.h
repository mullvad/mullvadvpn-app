#pragma once
#include <cstdint>

//
// WINFW public API
//

#ifdef WINROUTE_EXPORTS
#define WINROUTE_LINKAGE __declspec(dllexport)
#else
#define WINROUTE_LINKAGE __declspec(dllimport)
#endif

#define WINROUTE_API __stdcall

// Callback for logging.
typedef void (WINROUTE_API *WinRouteErrorSink)(const char *errorMessage, void *context);


extern "C"
WINROUTE_LINKAGE
bool
WINROUTE_API
WinRoute_EnsureTopMetric(
		const wchar_t *deviceAlias,
		WinRouteErrorSink errorSink,
	    void* errorSinkContext
);