#pragma once

#include <string>
#include <memory>
#include <vector>
#include <list>
#include <stdexcept>
#include <optional>
#include <mutex>
#include <winsock2.h>
#include <windows.h>
#include <ws2def.h>
#include <ws2ipdef.h>
#include <iphlpapi.h>
#include <netioapi.h>

// Custom header files below here.
// So broken networking headers don't get confused and break the compilation.
// ===
#include <libcommon/string.h>
#include <libcommon/logging/ilogsink.h>

namespace routemanager {

using Network = IP_ADDRESS_PREFIX;
using NodeAddress = SOCKADDR_INET;

bool EqualAddress(const Network &lhs, const Network &rhs);
bool EqualAddress(const NodeAddress &lhs, const NodeAddress &rhs);

class Node
{
public:

	Node(const std::optional<std::wstring> &deviceName, const std::optional<NodeAddress> &gateway)
		: m_deviceName(deviceName)
		, m_gateway(gateway)
	{
		if (false == m_deviceName.has_value() && false == m_gateway.has_value())
		{
			throw std::runtime_error("Invalid node definition");
		}

		if (m_deviceName.has_value())
		{
			const auto trimmed = common::string::Trim<>(m_deviceName.value());

			if (trimmed.empty())
			{
				throw std::runtime_error("Invalid device name in node definition");
			}

			m_deviceName = std::move(trimmed);
		}
	}

	const std::optional<std::wstring> &deviceName() const
	{
		return m_deviceName;
	}

	const std::optional<NodeAddress> &gateway() const
	{
		return m_gateway;
	}

	bool operator==(const Node &rhs) const
	{
		if (m_deviceName.has_value())
		{
			if (false == rhs.m_deviceName.has_value()
				|| 0 != _wcsicmp(m_deviceName.value().c_str(), rhs.deviceName().value().c_str()))
			{
				return false;
			}
		}

		if (m_gateway.has_value())
		{
			if (false == rhs.m_gateway.has_value()
				|| false == EqualAddress(m_gateway.value(), rhs.gateway().value()))
			{
				return false;
			}
		}

		return true;
	}

private:

	std::optional<std::wstring> m_deviceName;
	std::optional<NodeAddress> m_gateway;
};

class Route
{
public:

	Route(const Network &network, const std::optional<Node> &node)
		: m_network(network)
		, m_node(node)
	{
	}

	const Network &network() const
	{
		return m_network;
	}

	const std::optional<Node> &node() const
	{
		return m_node;
	}

	bool operator==(const Route &rhs) const
	{
		if (m_node.has_value())
		{
			return rhs.node().has_value()
				&& EqualAddress(m_network, rhs.network())
				&& m_node.value() == rhs.node().value();
		}

		return false == rhs.node().has_value()
			&& EqualAddress(m_network, rhs.network());
	}

private:

	Network m_network;
	std::optional<Node> m_node;
};

class RouteManager
{
public:

	RouteManager(std::shared_ptr<common::logging::ILogSink> logSink);
	~RouteManager();

	RouteManager(const RouteManager &) = delete;
	RouteManager &operator=(const RouteManager &) = delete;
	RouteManager(RouteManager &&) = default;

	void addRoutes(const std::vector<Route> &routes);
	void addRoute(const Route &route);

	void deleteRoutes(const std::vector<Route> &routes);
	void deleteRoute(const Route &route);

private:

	std::shared_ptr<common::logging::ILogSink> m_logSink;

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

	std::recursive_mutex m_routesLock;

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

	HANDLE m_notificationHandle;

	static void NETIOAPI_API_ RouteChangeCallback(void *context, MIB_IPFORWARD_ROW2 *row, MIB_NOTIFICATION_TYPE notificationType);
	void routeChangeCallback(MIB_IPFORWARD_ROW2 *row, MIB_NOTIFICATION_TYPE notificationType);

	static std::wstring FormatRegisteredRoute(const RegisteredRoute &route);
};

}
