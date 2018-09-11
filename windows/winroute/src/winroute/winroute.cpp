#include "stdafx.h"
#include "winroute.h"
#include "NetworkInterfaces.h"
#include "libcommon/error.h"
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

extern "C"
WINROUTE_LINKAGE
TAP_IPV6_STATUS
WINROUTE_API
GetTapInterfaceIpv6Status(
	WinRouteErrorSink errorSink,
	void* errorSinkContext
)
{
	try
	{
		MIB_IPINTERFACE_ROW interface = { 0 };

		interface.InterfaceLuid = NetworkInterfaces::GetInterfaceLuid(L"Mullvad");
		interface.Family = AF_INET6;

		const auto status = GetIpInterfaceEntry(&interface);

		if (NO_ERROR == status)
		{
			return TAP_IPV6_STATUS::ENABLED;
		}

		if (ERROR_NOT_FOUND == status)
		{
			return TAP_IPV6_STATUS::DISABLED;
		}

		common::error::Throw("Resolve TAP IPv6 interface", status);
	}
	catch (std::exception &err)
	{
		if (nullptr != errorSink)
		{
			errorSink(err.what(), errorSinkContext);
		}

		return TAP_IPV6_STATUS::FAILURE;
	}
	catch (...)
	{
		return TAP_IPV6_STATUS::FAILURE;
	}
}
