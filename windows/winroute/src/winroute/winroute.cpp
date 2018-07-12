#include "stdafx.h"
#include "winroute.h"
#include "NetworkInterfaces.h"
#include <cstdint>
#include <stdexcept>


extern "C"
WINROUTE_LINKAGE
WINROUTE_STATUS
WINROUTE_API
WinRoute_EnsureTopMetric(
	const wchar_t *deviceAlias,
	WinRouteErrorSink errorSink,
	void* errorSinkContext
) {
	try
	{
		NetworkInterfaces interfaces;
		bool metrics_set = interfaces.SetTopMetricForInterfacesByAlias(deviceAlias);
		return metrics_set ? WINROUTE_STATUS::METRIC_SET : WINROUTE_STATUS::METRIC_NO_CHANGE;
	}
	catch (std::exception &err)
	{
		if (nullptr != errorSink)
		{
			errorSink(err.what(), errorSinkContext);
		}
		return WINROUTE_STATUS::FAILURE;

	}
	catch (...)
	{
		return WINROUTE_STATUS::FAILURE;
	}
};

