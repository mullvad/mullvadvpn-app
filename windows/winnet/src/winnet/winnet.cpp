#include "stdafx.h"
#include "winnet.h"
#include "NetworkInterfaces.h"
#include "interfaceutils.h"
#include "netmonitor.h"
#include "../../shared/logsinkadapter.h"
#include <libcommon/error.h>
#include <cstdint>
#include <stdexcept>
#include <memory>

namespace
{

NetMonitor *g_NetMonitor = nullptr;

void UnwindAndLog(MullvadLogSink logSink, void *logSinkContext, const std::exception &err)
{
	if (nullptr == logSink)
	{
		return;
	}

	auto logger = std::make_shared<shared::LogSinkAdapter>(logSink, logSinkContext);

	common::error::UnwindException(err, logger);
}

} //anonymous namespace

extern "C"
WINNET_LINKAGE
WINNET_ETM_STATUS
WINNET_API
WinNet_EnsureTopMetric(
	const wchar_t *deviceAlias,
	MullvadLogSink logSink,
	void *logSinkContext
)
{
	try
	{
		NetworkInterfaces interfaces;
		bool metrics_set = interfaces.SetTopMetricForInterfacesByAlias(deviceAlias);
		return metrics_set ? WINNET_ETM_STATUS_METRIC_SET : WINNET_ETM_STATUS_METRIC_NO_CHANGE;
	}
	catch (const std::exception &err)
	{
		UnwindAndLog(logSink, logSinkContext, err);
		return WINNET_ETM_STATUS_FAILURE;
	}
	catch (...)
	{
		return WINNET_ETM_STATUS_FAILURE;
	}
};

extern "C"
WINNET_LINKAGE
WINNET_GTII_STATUS
WINNET_API
WinNet_GetTapInterfaceIpv6Status(
	MullvadLogSink logSink,
	void *logSinkContext
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
			return WINNET_GTII_STATUS_ENABLED;
		}

		if (ERROR_NOT_FOUND == status)
		{
			return WINNET_GTII_STATUS_DISABLED;
		}

		common::error::Throw("Resolve TAP IPv6 interface", status);
	}
	catch (const std::exception &err)
	{
		UnwindAndLog(logSink, logSinkContext, err);
		return WINNET_GTII_STATUS_FAILURE;
	}
	catch (...)
	{
		return WINNET_GTII_STATUS_FAILURE;
	}
}

extern "C"
WINNET_LINKAGE
bool
WINNET_API
WinNet_GetTapInterfaceAlias(
	wchar_t **alias,
	MullvadLogSink logSink,
	void *logSinkContext
)
{
	try
	{
		const auto currentAlias = InterfaceUtils::GetTapInterfaceAlias();

		auto stringBuffer = new wchar_t[currentAlias.size() + 1];
		wcscpy(stringBuffer, currentAlias.c_str());

		*alias = stringBuffer;

		return true;
	}
	catch (const std::exception &err)
	{
		UnwindAndLog(logSink, logSinkContext, err);
		return false;
	}
	catch (...)
	{
		return false;
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
bool
WINNET_API
WinNet_ActivateConnectivityMonitor(
	WinNetConnectivityMonitorCallback callback,
	void *callbackContext,
	bool *currentConnectivity,
	MullvadLogSink logSink,
	void *logSinkContext
)
{
	try
	{
		if (nullptr != g_NetMonitor)
		{
			throw std::runtime_error("Cannot activate connectivity monitor twice");
		}

		auto forwarder = [callback, callbackContext](bool connected)
		{
			callback(connected, callbackContext);
		};

		bool connected = false;

		auto logger = std::make_shared<shared::LogSinkAdapter>(logSink, logSinkContext);

		g_NetMonitor = new NetMonitor(logger, forwarder, connected);

		if (nullptr != currentConnectivity)
		{
			*currentConnectivity = connected;
		}

		return true;
	}
	catch (const std::exception &err)
	{
		UnwindAndLog(logSink, logSinkContext, err);
		return false;
	}
	catch (...)
	{
		return false;
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

extern "C"
WINNET_LINKAGE
WINNET_CC_STATUS
WINNET_API
WinNet_CheckConnectivity(
	MullvadLogSink logSink,
	void *logSinkContext
)
{
	try
	{
		return (NetMonitor::CheckConnectivity(std::make_shared<shared::LogSinkAdapter>(logSink, logSinkContext))
			? WINNET_CC_STATUS_CONNECTED : WINNET_CC_STATUS_NOT_CONNECTED);
	}
	catch (const std::exception &err)
	{
		UnwindAndLog(logSink, logSinkContext, err);
		return WINNET_CC_STATUS_CONNECTIVITY_UNKNOWN;
	}
	catch (...)
	{
		return WINNET_CC_STATUS_CONNECTIVITY_UNKNOWN;
	}
}
