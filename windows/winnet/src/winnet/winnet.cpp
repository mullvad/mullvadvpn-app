#include "stdafx.h"
#include "winnet.h"
#include "NetworkInterfaces.h"
#include "interfaceutils.h"
#include "libcommon/error.h"
#include <cstdint>
#include <stdexcept>

extern "C"
WINNET_LINKAGE
WINNET_ETM_STATUS
WINNET_API
WinNet_EnsureTopMetric(
	const wchar_t *deviceAlias,
	WinNetErrorSink errorSink,
	void* errorSinkContext
)
{
	try
	{
		NetworkInterfaces interfaces;
		bool metrics_set = interfaces.SetTopMetricForInterfacesByAlias(deviceAlias);
		return metrics_set ? WINNET_ETM_STATUS::METRIC_SET : WINNET_ETM_STATUS::METRIC_NO_CHANGE;
	}
	catch (std::exception &err)
	{
		if (nullptr != errorSink)
		{
			errorSink(err.what(), errorSinkContext);
		}

		return WINNET_ETM_STATUS::FAILURE;
	}
	catch (...)
	{
		return WINNET_ETM_STATUS::FAILURE;
	}
};

extern "C"
WINNET_LINKAGE
WINNET_GTII_STATUS
WINNET_API
WinNet_GetTapInterfaceIpv6Status(
	WinNetErrorSink errorSink,
	void* errorSinkContext
)
{
	try
	{
		MIB_IPINTERFACE_ROW interface = { 0 };

		interface.InterfaceLuid = NetworkInterfaces::GetInterfaceLuid(InterfaceUtils::GetTapInterfaceAlias());
		interface.Family = AF_INET6;

		const auto status = GetIpInterfaceEntry(&interface);

		if (NO_ERROR == status)
		{
			return WINNET_GTII_STATUS::ENABLED;
		}

		if (ERROR_NOT_FOUND == status)
		{
			return WINNET_GTII_STATUS::DISABLED;
		}

		common::error::Throw("Resolve TAP IPv6 interface", status);
	}
	catch (std::exception &err)
	{
		if (nullptr != errorSink)
		{
			errorSink(err.what(), errorSinkContext);
		}

		return WINNET_GTII_STATUS::FAILURE;
	}
	catch (...)
	{
		return WINNET_GTII_STATUS::FAILURE;
	}
}

extern "C"
WINNET_LINKAGE
WINNET_GTIA_STATUS
WINNET_API
WinNet_GetTapInterfaceAlias(
	wchar_t **alias,
	WinNetErrorSink errorSink,
	void* errorSinkContext
)
{
	try
	{
		const auto currentAlias = InterfaceUtils::GetTapInterfaceAlias();

		auto stringBuffer = new wchar_t[currentAlias.size() + 1];
		wcscpy(stringBuffer, currentAlias.c_str());

		*alias = stringBuffer;

		return WINNET_GTIA_STATUS::SUCCESS;
	}
	catch (std::exception &err)
	{
		if (nullptr != errorSink)
		{
			errorSink(err.what(), errorSinkContext);
		}

		return WINNET_GTIA_STATUS::FAILURE;
	}
	catch (...)
	{
		return WINNET_GTIA_STATUS::FAILURE;
	}
}

extern "C"
WINNET_LINKAGE
void
WINNET_API
WinNet_ReleaseString(
	wchar_t *str
)
{
	try
	{
		delete[] str;
	}
	catch (...)
	{
	}
}
