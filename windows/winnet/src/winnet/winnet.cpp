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
#include <libcommon/memory.h>
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
WINNET_EBM_STATUS
WINNET_API
WinNet_EnsureBestMetric(
	const wchar_t *deviceAlias,
	MullvadLogSink logSink,
	void *logSinkContext
)
{
	try
	{
		if (nullptr == deviceAlias)
		{
			THROW_ERROR("Invalid argument: deviceAlias");
		}

		NetworkInterfaces interfaces;
		return interfaces.SetBestMetricForInterfacesByAlias(deviceAlias) ?
			WINNET_EBM_STATUS_METRIC_SET : WINNET_EBM_STATUS_METRIC_NO_CHANGE;
	}
	catch (const std::exception &err)
	{
		shared::logging::UnwindAndLog(logSink, logSinkContext, err);
		return WINNET_EBM_STATUS_FAILURE;
	}
	catch (...)
	{
		return WINNET_EBM_STATUS_FAILURE;
	}
};

extern "C"
WINNET_LINKAGE
WINNET_STATUS
WINNET_API
WinNet_GetBestDefaultRoute(
	WINNET_ADDR_FAMILY family,
	WINNET_DEFAULT_ROUTE *route,
	MullvadLogSink logSink,
	void *logSinkContext
)
{
	try
	{
		if (nullptr == route)
		{
			THROW_ERROR("Invalid argument: route");
		}

		static const std::pair<WINNET_ADDR_FAMILY, ADDRESS_FAMILY> familyMap[] =
		{
			{ WINNET_ADDR_FAMILY_IPV4, static_cast<ADDRESS_FAMILY>(AF_INET) },
			{ WINNET_ADDR_FAMILY_IPV6, static_cast<ADDRESS_FAMILY>(AF_INET6) }
		};
		const auto win_family = common::ValueMapper::Map<>(family, familyMap);

		const auto ifaceAndGateway = GetBestDefaultRoute(win_family);

		if (!ifaceAndGateway.has_value())
		{
			return WINNET_STATUS_NOT_FOUND;
		}

		route->interfaceLuid = ifaceAndGateway->iface.Value;
		const auto ips = winnet::ConvertNativeAddresses(&ifaceAndGateway->gateway, 1);
		route->gateway = ips[0];

		return WINNET_STATUS_SUCCESS;
	}
	catch (const std::exception & err)
	{
		shared::logging::UnwindAndLog(logSink, logSinkContext, err);
		return WINNET_STATUS_FAILURE;
	}
	catch (...)
	{
		return WINNET_STATUS_FAILURE;
	}
}

extern "C"
WINNET_LINKAGE
WINNET_STATUS
WINNET_API
WinNet_InterfaceLuidToIpAddress(
	WINNET_ADDR_FAMILY family,
	uint64_t interfaceLuid,
	WINNET_IP *ip,
	MullvadLogSink logSink,
	void *logSinkContext
)
{
	try
	{
		if (nullptr == ip)
		{
			THROW_ERROR("Invalid argument: ip");
		}

		static const std::pair<WINNET_ADDR_FAMILY, ADDRESS_FAMILY> familyMap[] =
		{
			{ WINNET_ADDR_FAMILY_IPV4, static_cast<ADDRESS_FAMILY>(AF_INET) },
			{ WINNET_ADDR_FAMILY_IPV6, static_cast<ADDRESS_FAMILY>(AF_INET6) }
		};
		const auto win_family = common::ValueMapper::Map<>(family, familyMap);

		MIB_UNICASTIPADDRESS_TABLE *table = nullptr;
		const auto status = GetUnicastIpAddressTable(win_family, &table);

		if (NO_ERROR != status)
		{
			THROW_WINDOWS_ERROR(status, "GetUnicastIpAddressTable");
		}

		common::memory::ScopeDestructor destructor;

		destructor += [table]() {
			FreeMibTable(table);
		};

		for (ULONG i = 0; i < table->NumEntries; i++)
		{
			const auto entry = table->Table[i];

			if (interfaceLuid != entry.InterfaceLuid.Value)
			{
				continue;
			}

			// Found IP address
			const auto ips = winnet::ConvertNativeAddresses(&entry.Address, 1);
			*ip = ips[0];

			return WINNET_STATUS_SUCCESS;
		}

		return WINNET_STATUS_NOT_FOUND;
	}
	catch (const std::exception & err)
	{
		shared::logging::UnwindAndLog(logSink, logSinkContext, err);
		return WINNET_STATUS_FAILURE;
	}
	catch (...)
	{
		return WINNET_STATUS_FAILURE;
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

		if (nullptr == callback)
		{
			THROW_ERROR("Invalid argument: callback");
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
WINNET_AR_STATUS
WINNET_API
WinNet_AddRoutes(
	const WINNET_ROUTE *routes,
	uint32_t numRoutes
)
{
	AutoLockType lock(g_RouteManagerLock);

	if (nullptr == g_RouteManager)
	{
		return WINNET_AR_STATUS_GENERAL_ERROR;
	}

	try
	{
		if (nullptr == routes)
		{
			THROW_ERROR("Invalid argument: routes");
		}

		g_RouteManager->addRoutes(winnet::ConvertRoutes(routes, numRoutes));
		return WINNET_AR_STATUS_SUCCESS;
	}
	catch (const winnet::routing::error::NoDefaultRoute &err)
	{
		common::error::UnwindException(err, g_RouteManagerLogSink);
		return WINNET_AR_STATUS_NO_DEFAULT_ROUTE;
	}
	catch (const winnet::routing::error::DeviceNameNotFound &err)
	{
		common::error::UnwindException(err, g_RouteManagerLogSink);
		return WINNET_AR_STATUS_NAME_NOT_FOUND;
	}
	catch (const winnet::routing::error::DeviceGatewayNotFound &err)
	{
		common::error::UnwindException(err, g_RouteManagerLogSink);
		return WINNET_AR_STATUS_GATEWAY_NOT_FOUND;
	}
	catch (const std::exception &err)
	{
		common::error::UnwindException(err, g_RouteManagerLogSink);
		return WINNET_AR_STATUS_GENERAL_ERROR;
	}
	catch (...)
	{
		return WINNET_AR_STATUS_GENERAL_ERROR;
	}
}

extern "C"
WINNET_LINKAGE
WINNET_AR_STATUS
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
		if (nullptr == routes)
		{
			THROW_ERROR("Invalid argument: routes");
		}

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
WinNet_DeleteAppliedRoutes()
{
	AutoLockType lock(g_RouteManagerLock);

	if (nullptr == g_RouteManager)
	{
		return false;
	}

	try
	{
		g_RouteManager->deleteAppliedRoutes();
		return true;
	}
	catch (const std::exception & err)
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
		if (nullptr == callback)
		{
			THROW_ERROR("Invalid argument: callback");
		}

		if (nullptr == registrationHandle)
		{
			THROW_ERROR("Invalid argument: registrationHandle");
		}

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

			static const std::pair<ADDRESS_FAMILY, WINNET_ADDR_FAMILY> familyMap[] =
			{
				{ static_cast<ADDRESS_FAMILY>(AF_INET), WINNET_ADDR_FAMILY_IPV4 },
				{ static_cast<ADDRESS_FAMILY>(AF_INET6), WINNET_ADDR_FAMILY_IPV6 }
			};

			const auto translatedFamily = common::ValueMapper::Map<>(family, familyMap);

			WINNET_DEFAULT_ROUTE defaultRoute = { 0 };

			//
			// Determine which LUID and gateway to forward.
			//

			if (RouteManager::DefaultRouteChangedEventType::Updated == eventType)
			{
				const auto ips = winnet::ConvertNativeAddresses(&route.value().gateway, 1);
				defaultRoute.gateway = ips[0];
				defaultRoute.interfaceLuid = route.value().iface.Value;
			}

			//
			// Forward to client.
			//

			callback(translatedEventType, translatedFamily, defaultRoute, context);
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

		g_RouteManagerLogSink.reset();
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
		if (nullptr == deviceAlias)
		{
			THROW_ERROR("Invalid argument: deviceAlias")
		}

		if (nullptr == addresses)
		{
			THROW_ERROR("Invalid argument: addresses")
		}

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
