#include "stdafx.h"
#include "winnet.h"
#include "NetworkInterfaces.h"
#include "interfaceutils.h"
#include "libcommon/error.h"
#include "netmonitor.h"
#include <cstdint>
#include <stdexcept>

namespace
{

NetMonitor *g_NetMonitor = nullptr;

} //anonymous namespace

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

extern "C"
WINNET_LINKAGE
WINNET_ACM_STATUS
WINNET_API
WinNet_ActivateConnectivityMonitor(
	WinNetConnectivityMonitorCallback callback,
	uint8_t *currentConnectivity,
	WinNetErrorSink errorSink,
	void* errorSinkContext
)
{
	try
	{
		if (nullptr != g_NetMonitor)
		{
			throw std::runtime_error("Cannot activate connectivity monitor twice");
		}

		auto forwarder = [callback](bool connected)
		{
			callback(static_cast<uint8_t>(connected));
		};

		bool connected = false;

		g_NetMonitor = new NetMonitor(forwarder, connected);

		if (nullptr != currentConnectivity)
		{
			*currentConnectivity = static_cast<uint8_t>(connected);
		}

		return WINNET_ACM_STATUS::SUCCESS;
	}
	catch (std::exception &err)
	{
		if (nullptr != errorSink)
		{
			errorSink(err.what(), errorSinkContext);
		}

		return WINNET_ACM_STATUS::FAILURE;
	}
	catch (...)
	{
		return WINNET_ACM_STATUS::FAILURE;
	}
}

extern "C"
WINNET_LINKAGE
void
WINNET_API
WinNet_DeactivateConnectivityMonitor(
)
{
	try
	{
		delete g_NetMonitor;
		g_NetMonitor = nullptr;
	}
	catch (...)
	{
	}
}
