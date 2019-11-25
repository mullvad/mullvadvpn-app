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

namespace winnet::routing
{

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
	void addRoute(const Route &route);

	void deleteRoutes(const std::vector<Route> &routes);
	void deleteRoute(const Route &route);

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

	// Find record based on destination and mask.
	std::list<RouteRecord>::iterator findRouteRecord(const Network &network);

	// Note: Same as above!
	std::list<RouteRecord>::iterator findRouteRecord(const Route &route);

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
