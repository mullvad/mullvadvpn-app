#pragma once

#include <string>
#include <memory>
#include <vector>
#include <list>
#include <optional>
#include <mutex>
#include <functional>
#include <windows.h>
#include <ws2def.h>
#include <ifdef.h>
#include <libcommon/string.h>
#include <libcommon/logging/ilogsink.h>
#include "defaultroutemonitor.h"
#include "helpers.h"

namespace winnet::routing
{

namespace error
{

class RouteManagerError : public std::runtime_error
{
public:

	RouteManagerError(const char* message)
		: std::runtime_error(message)
	{
	}
};

class NoDefaultRoute : public RouteManagerError
{
public:

	NoDefaultRoute(const char* message)
		: RouteManagerError(message)
	{
	}
};

class DeviceNameNotFound : public RouteManagerError
{
public:

	DeviceNameNotFound(const char* message)
		: RouteManagerError(message)
	{
	}
};

class DeviceGatewayNotFound : public RouteManagerError
{
public:

	DeviceGatewayNotFound(const char* message)
		: RouteManagerError(message)
	{
	}
};

}

class RouteManager
{
public:

	RouteManager(std::shared_ptr<common::logging::ILogSink> logSink);
	~RouteManager();

	RouteManager(const RouteManager &) = delete;
	RouteManager(RouteManager &&) = default;
	RouteManager &operator=(const RouteManager &) = delete;
	RouteManager &operator=(RouteManager &&) = delete;
	
	void addRoutes(const std::vector<Route> &routes);
	void deleteRoutes(const std::vector<Route> &routes);
	void deleteAppliedRoutes();

	using DefaultRouteChangedEventType = DefaultRouteMonitor::EventType;

	using DefaultRouteChangedCallback = std::function<void
	(
		DefaultRouteChangedEventType eventType,
		ADDRESS_FAMILY family,

		// For update events, data associated with the new best default route.
		const std::optional<InterfaceAndGateway> &route
	)>;

	using CallbackHandle = void*;

	CallbackHandle registerDefaultRouteChangedCallback(DefaultRouteChangedCallback callback);
	void unregisterDefaultRouteChangedCallback(CallbackHandle handle);

private:

	std::shared_ptr<common::logging::ILogSink> m_logSink;

	std::unique_ptr<DefaultRouteMonitor> m_routeMonitorV4;
	std::unique_ptr<DefaultRouteMonitor> m_routeMonitorV6;

	// These are the exact details derived from the route specification (`Route`).
	// They are used when registering and deleting a route in the system.
	struct RegisteredRoute
	{
		Network network;
		NET_LUID luid;
		NodeAddress nextHop;

		bool operator==(const RegisteredRoute &rhs) const
		{
			return luid.Value == rhs.luid.Value
				&& EqualAddress(nextHop, rhs.nextHop)
				&& EqualAddress(network, rhs.network);
		}
	};

	struct RouteRecord
	{
		Route route;
		RegisteredRoute registeredRoute;
	};

	std::list<RouteRecord> m_routes;
	std::mutex m_routesLock;

	std::list<DefaultRouteChangedCallback> m_defaultRouteCallbacks;
	std::recursive_mutex m_defaultRouteCallbacksLock;

	//
	// Find record based on route registration data.
	//
	// Note: Searching the records and matching on route specification is
	// unreliable because of the node attribute on the route. Different node
	// specifications can resolve to the same physical node.
	//
	// (node = exit node = interface)
	//
	std::list<RouteRecord>::iterator findRouteRecord(const RegisteredRoute &route);

	//
	// Find record based on route specification.
	//
	// Note: Only ever use this to find the registration data for a route
	// that was successfully registered previously.
	//
	std::list<RouteRecord>::iterator findRouteRecordFromSpec(const Route &route);

	RegisteredRoute addIntoRoutingTable(const Route &route);
	void restoreIntoRoutingTable(const RegisteredRoute &route);
	void deleteFromRoutingTable(const RegisteredRoute &route);

	enum class EventType
	{
		ADD_ROUTE,
		DELETE_ROUTE,
	};

	struct EventEntry
	{
		EventType type;
		RouteRecord record;
	};

	void undoEvents(const std::vector<EventEntry> &eventLog);

	static std::wstring FormatRegisteredRoute(const RegisteredRoute &route);

	void defaultRouteChanged(ADDRESS_FAMILY family, DefaultRouteMonitor::EventType eventType,
		const std::optional<InterfaceAndGateway> &route);
};

}
