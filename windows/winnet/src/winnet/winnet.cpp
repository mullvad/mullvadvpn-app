#include "stdafx.h"
#include "winnet.h"
#include "NetworkInterfaces.h"
#include "interfaceutils.h"
#include "offlinemonitor.h"
#include "routing/routemanager.h"
#include "../../shared/logsinkadapter.h"
#include <libcommon/error.h>
#include <libcommon/network.h>
#include <cstdint>
#include <stdexcept>
#include <memory>
#include <optional>

using namespace winnet::routing;

namespace
{

OfflineMonitor *g_OfflineMonitor = nullptr;

RouteManager *g_RouteManager = nullptr;
std::shared_ptr<shared::LogSinkAdapter> g_RouteManagerLogSink;

Network ConvertNetwork(const WINNET_IPNETWORK &in)
{
	//
	// Convert WINNET_IPNETWORK into Network aka IP_ADDRESS_PREFIX
	//

	Network out{ 0 };

	out.PrefixLength = in.prefix;

	switch (in.type)
	{
		case WINNET_IP_TYPE_IPV4:
		{
			out.Prefix.si_family = AF_INET;
			out.Prefix.Ipv4.sin_family = AF_INET;
			out.Prefix.Ipv4.sin_addr.s_addr = *reinterpret_cast<const uint32_t *>(in.bytes);

			break;
		}
		case WINNET_IP_TYPE_IPV6:
		{
			out.Prefix.si_family = AF_INET6;
			out.Prefix.Ipv6.sin6_family = AF_INET6;
			memcpy(out.Prefix.Ipv6.sin6_addr.u.Byte, in.bytes, 16);

			break;
		}
		default:
		{
			throw std::runtime_error("Missing case handler in switch clause");
		}
	}

	return out;
}

std::optional<Node> ConvertNode(const WINNET_NODE *in)
{
	if (nullptr == in)
	{
		return {};
	}

	if (nullptr == in->deviceName && nullptr == in->gateway)
	{
		throw std::runtime_error("Invalid 'WINNET_NODE' definition");
	}

	std::optional<std::wstring> deviceName;
	std::optional<NodeAddress> gateway;

	if (nullptr != in->deviceName)
	{
		deviceName = in->deviceName;
	}

	if (nullptr != in->gateway)
	{
		NodeAddress gw { 0 };

		switch (in->gateway->type)
		{
			case WINNET_IP_TYPE_IPV4:
			{
				gw.si_family = AF_INET;
				gw.Ipv4.sin_addr.s_addr = *reinterpret_cast<const uint32_t *>(in->gateway->bytes);

				break;
			}
			case WINNET_IP_TYPE_IPV6:
			{
				gw.si_family = AF_INET6;
				memcpy(&gw.Ipv6.sin6_addr.u.Byte, in->gateway->bytes, 16);

				break;
			}
			default:
			{
				throw std::logic_error("Invalid gateway type specifier in 'WINNET_NODE' definition");
			}
		}

		gateway = gw;
	}

	return Node(deviceName, gateway);
}

std::vector<Route> ConvertRoutes(const WINNET_ROUTE *routes, uint32_t numRoutes)
{
	std::vector<Route> out;

	out.reserve(numRoutes);

	for (size_t i = 0; i < numRoutes; ++i)
	{
		out.emplace_back(Route
		{
			ConvertNetwork(routes[i].network),
			ConvertNode(routes[i].node)
		});
	}

	return out;
}

void UnwindAndLog(MullvadLogSink logSink, void *logSinkContext, const std::exception &err)
{
	if (nullptr == logSink)
	{
		return;
	}

	auto logger = std::make_shared<shared::LogSinkAdapter>(logSink, logSinkContext);

	common::error::UnwindException(err, logger);
}

std::vector<SOCKADDR_INET> ConvertAddresses(const WINNET_IP *addresses, uint32_t numAddresses)
{
	//
	// This duplicates the same logic we have above.
	// TODO: Fix when time permits.
	//

	std::vector<SOCKADDR_INET> out;
	out.reserve(numAddresses);

	for (uint32_t i = 0; i < numAddresses; ++i)
	{
		const WINNET_IP &from = addresses[i];
		SOCKADDR_INET to{ 0 };

		switch (from.type)
		{
			case WINNET_IP_TYPE_IPV4:
			{
				to.si_family = AF_INET;
				to.Ipv4.sin_addr.s_addr = *reinterpret_cast<const uint32_t *>(from.bytes);

				break;
			}
			case WINNET_IP_TYPE_IPV6:
			{
				to.si_family = AF_INET6;
				memcpy(&to.Ipv6.sin6_addr.u.Byte, from.bytes, 16);

				break;
			}
			default:
			{
				throw std::logic_error("Invalid address family in 'WINNET_IP' definition");
			}
		 }

		 out.push_back(to);
	}

	return out;
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
	MullvadLogSink logSink,
	void *logSinkContext
)
{
	try
	{
		if (nullptr != g_OfflineMonitor)
		{
			throw std::runtime_error("Cannot activate connectivity monitor twice");
		}

		auto forwarder = [callback, callbackContext](bool connected)
		{
			callback(connected, callbackContext);
		};

		auto logger = std::make_shared<shared::LogSinkAdapter>(logSink, logSinkContext);

		g_OfflineMonitor = new OfflineMonitor(logger, forwarder);

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
	try
	{
		if (nullptr != g_RouteManager)
		{
			throw std::runtime_error("Cannot activate route manager twice");
		}

		g_RouteManagerLogSink =   std::make_shared<shared::LogSinkAdapter>(logSink, logSinkContext);
		g_RouteManager = new RouteManager(g_RouteManagerLogSink);

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
bool
WINNET_API
WinNet_AddRoutes(
	const WINNET_ROUTE *routes,
	uint32_t numRoutes
)
{
	if (nullptr == g_RouteManager)
	{
		return false;
	}

	try
	{
		g_RouteManager->addRoutes(ConvertRoutes(routes, numRoutes));
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
	if (nullptr == g_RouteManager)
	{
		return false;
	}

	try
	{
		g_RouteManager->addRoute
		(
			Route{ ConvertNetwork(route->network), ConvertNode(route->node) }
		);

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
WinNet_DeleteRoutes(
	const WINNET_ROUTE *routes,
	uint32_t numRoutes
)
{
	if (nullptr == g_RouteManager)
	{
		return false;
	}

	try
	{
		g_RouteManager->deleteRoutes(ConvertRoutes(routes, numRoutes));
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
	if (nullptr == g_RouteManager)
	{
		return false;
	}

	try
	{
		g_RouteManager->deleteRoute
		(
			Route{ ConvertNetwork(route->network), ConvertNode(route->node) }
		);

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

//
// TODO: Move to libcommon.
//
struct ValueMapper
{
	template<typename T, typename U, std::size_t S>
	static U map(T t, const std::pair<T, U> (&dictionary)[S])
	{
		for (const auto &entry : dictionary)
		{
			if (t == entry.first)
			{
				return entry.second;
			}
		}

		throw std::runtime_error("Could not map between values");
	}
};

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

			const auto translatedEventType = ValueMapper::map<>(eventType, eventTypeMap);

			//
			// Translate the family type.
			//

			static const std::pair<ADDRESS_FAMILY, WINNET_IP_FAMILY> familyMap[] =
			{
				{ static_cast<ADDRESS_FAMILY>(AF_INET), WINNET_IP_FAMILY_V4 },
				{ static_cast<ADDRESS_FAMILY>(AF_INET6), WINNET_IP_FAMILY_V6 }
			};

			const auto translatedFamily = ValueMapper::map<>(family, familyMap);

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
			const auto ansiName = common::string::ToAnsi(deviceAlias);
			const auto err = std::string("Unable to derive interface LUID from interface alias: ").append(ansiName);

			throw std::runtime_error(err);
		}

		InterfaceUtils::AddDeviceIpAddresses(luid, ConvertAddresses(addresses, numAddresses));

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
