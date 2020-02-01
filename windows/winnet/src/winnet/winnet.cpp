#include "stdafx.h"
#include "winnet.h"
#include "NetworkInterfaces.h"
#include "offlinemonitor.h"
#include "routing/routemanager.h"
#include "converters.h"
#include <libshared/logging/logsinkadapter.h>
#include <libshared/logging/unwind.h>
#include <libshared/network/interfaceutils.h>
#include <libcommon/error.h>
#include <libcommon/valuemapper.h>
#include <libcommon/network.h>
#include <cstdint>
#include <memory>
#include <optional>
#include <mutex>

using namespace winnet::routing;
using namespace common::network;
using AutoLockType = std::scoped_lock<std::mutex>;
using namespace shared::network;

namespace
{

OfflineMonitor *g_OfflineMonitor = nullptr;

std::mutex g_RouteManagerLock;
RouteManager *g_RouteManager = nullptr;
std::shared_ptr<shared::logging::LogSinkAdapter> g_RouteManagerLogSink;

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
		shared::logging::UnwindAndLog(logSink, logSinkContext, err);
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
		MIB_IPINTERFACE_ROW iface = { 0 };

		iface.InterfaceLuid = NetworkInterfaces::GetInterfaceLuid(InterfaceUtils::GetTapInterfaceAlias());
		iface.Family = AF_INET6;

		const auto status = GetIpInterfaceEntry(&iface);

		if (NO_ERROR == status)
		{
			return WINNET_GTII_STATUS_ENABLED;
		}

		if (ERROR_NOT_FOUND == status)
		{
			return WINNET_GTII_STATUS_DISABLED;
		}

		THROW_WINDOWS_ERROR(status, "Resolve TAP IPv6 interface");
	}
	catch (const std::exception &err)
	{
		shared::logging::UnwindAndLog(logSink, logSinkContext, err);
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
		shared::logging::UnwindAndLog(logSink, logSinkContext, err);
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
	MullvadLogSink logSink,
	void *logSinkContext
)
{
	try
	{
		if (nullptr != g_OfflineMonitor)
		{
			THROW_ERROR("Cannot activate connectivity monitor twice");
		}

		auto forwarder = [callback, callbackContext](bool connected)
		{
			callback(connected, callbackContext);
		};

		auto logger = std::make_shared<shared::logging::LogSinkAdapter>(logSink, logSinkContext);

		g_OfflineMonitor = new OfflineMonitor(logger, forwarder);

		return true;
	}
	catch (const std::exception &err)
	{
		shared::logging::UnwindAndLog(logSink, logSinkContext, err);
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
		delete g_OfflineMonitor;
		g_OfflineMonitor = nullptr;
	}
	catch (...)
	{
	}
}

extern "C"
WINNET_LINKAGE
bool
WINNET_API
WinNet_ActivateRouteManager(
	MullvadLogSink logSink,
	void *logSinkContext
)
{
	AutoLockType lock(g_RouteManagerLock);

	try
	{
		if (nullptr != g_RouteManager)
		{
			THROW_ERROR("Cannot activate route manager twice");
		}

		g_RouteManagerLogSink = std::make_shared<shared::logging::LogSinkAdapter>(logSink, logSinkContext);
		g_RouteManager = new RouteManager(g_RouteManagerLogSink);

		return true;
	}
	catch (const std::exception &err)
	{
		shared::logging::UnwindAndLog(logSink, logSinkContext, err);
		return false;
	}
	catch (...)
	{
		return false;
	}
}

extern "C"
WINNET_LINKAGE
bool
WINNET_API
WinNet_AddRoutes(
	const WINNET_ROUTE *routes,
	uint32_t numRoutes
)
{
	AutoLockType lock(g_RouteManagerLock);

	if (nullptr == g_RouteManager)
	{
		return false;
	}

	try
	{
		g_RouteManager->addRoutes(winnet::ConvertRoutes(routes, numRoutes));
		return true;
	}
	catch (const std::exception &err)
	{
		common::error::UnwindException(err, g_RouteManagerLogSink);
		return false;
	}
	catch (...)
	{
		return false;
	}
}

extern "C"
WINNET_LINKAGE
bool
WINNET_API
WinNet_AddRoute(
	const WINNET_ROUTE *route
)
{
	return WinNet_AddRoutes(route, 1);
}

extern "C"
WINNET_LINKAGE
bool
WINNET_API
WinNet_DeleteRoutes(
	const WINNET_ROUTE *routes,
	uint32_t numRoutes
)
{
	AutoLockType lock(g_RouteManagerLock);

	if (nullptr == g_RouteManager)
	{
		return false;
	}

	try
	{
		g_RouteManager->deleteRoutes(winnet::ConvertRoutes(routes, numRoutes));
		return true;
	}
	catch (const std::exception &err)
	{
		common::error::UnwindException(err, g_RouteManagerLogSink);
		return false;
	}
	catch (...)
	{
		return false;
	}
}

extern "C"
WINNET_LINKAGE
bool
WINNET_API
WinNet_DeleteRoute(
	const WINNET_ROUTE *route
)
{
	return WinNet_DeleteRoutes(route, 1);
}

extern "C"
WINNET_LINKAGE
bool
WINNET_API
WinNet_RegisterDefaultRouteChangedCallback(
	WinNetDefaultRouteChangedCallback callback,
	void *context,
	void **registrationHandle
)
{
	AutoLockType lock(g_RouteManagerLock);

	if (nullptr == g_RouteManager)
	{
		return false;
	}

	try
	{
		auto forwarder = [callback, context](RouteManager::DefaultRouteChangedEventType eventType,
			ADDRESS_FAMILY family, const std::optional<InterfaceAndGateway> &route)
		{
			//
			// Translate the event type.
			//

			using from_t = RouteManager::DefaultRouteChangedEventType;
			using to_t = WINNET_DEFAULT_ROUTE_CHANGED_EVENT_TYPE;

			static const std::pair<from_t, to_t> eventTypeMap[] =
			{
				{ from_t::Updated, WINNET_DEFAULT_ROUTE_CHANGED_EVENT_TYPE_UPDATED },
				{ from_t::Removed, WINNET_DEFAULT_ROUTE_CHANGED_EVENT_TYPE_REMOVED }
			};

			const auto translatedEventType = common::ValueMapper::Map<>(eventType, eventTypeMap);

			//
			// Translate the family type.
			//

			static const std::pair<ADDRESS_FAMILY, WINNET_IP_FAMILY> familyMap[] =
			{
				{ static_cast<ADDRESS_FAMILY>(AF_INET), WINNET_IP_FAMILY_V4 },
				{ static_cast<ADDRESS_FAMILY>(AF_INET6), WINNET_IP_FAMILY_V6 }
			};

			const auto translatedFamily = common::ValueMapper::Map<>(family, familyMap);

			//
			// Determine which LUID to forward.
			//

			uint64_t translatedLuid = 0;

			if (RouteManager::DefaultRouteChangedEventType::Updated == eventType)
			{
				translatedLuid = route.value().iface.Value;
			}

			//
			// Forward to client.
			//

			callback(translatedEventType, translatedFamily, translatedLuid, context);
		};

		*registrationHandle = g_RouteManager->registerDefaultRouteChangedCallback(forwarder);

		return true;
	}
	catch (const std::exception &err)
	{
		common::error::UnwindException(err, g_RouteManagerLogSink);
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
WinNet_UnregisterDefaultRouteChangedCallback(
	void *registrationHandle
)
{
	AutoLockType lock(g_RouteManagerLock);

	if (nullptr == g_RouteManager)
	{
		return;
	}

	try
	{
		g_RouteManager->unregisterDefaultRouteChangedCallback(registrationHandle);
	}
	catch (const std::exception &err)
	{
		g_RouteManagerLogSink->error("Failed to unregister default-route-changed callback");
		common::error::UnwindException(err, g_RouteManagerLogSink);
	}
	catch (...)
	{
	}
}

extern "C"
WINNET_LINKAGE
void
WINNET_API
WinNet_DeactivateRouteManager(
)
{
	AutoLockType lock(g_RouteManagerLock);

	try
	{
		delete g_RouteManager;
		g_RouteManager = nullptr;
	}
	catch (...)
	{
	}
}

extern "C"
WINNET_LINKAGE
bool
WINNET_API
WinNet_AddDeviceIpAddresses(
	const wchar_t *deviceAlias,
	const WINNET_IP *addresses,
	uint32_t numAddresses,
	MullvadLogSink logSink,
	void *logSinkContext
)
{
	try
	{
		NET_LUID luid;

		if (0 != ConvertInterfaceAliasToLuid(deviceAlias, &luid))
		{
			const auto msg = std::string("Unable to derive interface LUID from interface alias: ")
				.append(common::string::ToAnsi(deviceAlias));

			THROW_ERROR(msg.c_str());
		}

		InterfaceUtils::AddDeviceIpAddresses(luid, winnet::ConvertAddresses(addresses, numAddresses));

		return true;
	}
	catch (const std::exception &err)
	{
		shared::logging::UnwindAndLog(logSink, logSinkContext, err);
		return false;
	}
	catch (...)
	{
		return false;
	}
}
