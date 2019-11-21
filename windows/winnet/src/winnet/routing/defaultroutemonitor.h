#pragma once

#include <ifdef.h>
#include <ws2def.h>
#include <functional>
#include <optional>
#include <memory>
#include <mutex>
#include <libcommon/logging/ilogsink.h>
#include <libcommon/burstguard.h>
#include "types.h"

namespace winnet::routing
{

class DefaultRouteMonitor
{
public:

	enum class EventType
	{
		// The best default route changed.
		Updated,

		// No default routes exist.
		Removed,
	};

	using Callback = std::function<void
	(
		EventType eventType,

		// For update events, data associated with the new best default route.
		const std::optional<InterfaceAndGateway> &route
	)>;

	DefaultRouteMonitor(ADDRESS_FAMILY family, Callback callback, std::shared_ptr<common::logging::ILogSink> logSink);
	~DefaultRouteMonitor();

	DefaultRouteMonitor(const DefaultRouteMonitor &) = delete;
	DefaultRouteMonitor(DefaultRouteMonitor &&) = delete;
	DefaultRouteMonitor &operator=(const DefaultRouteMonitor &) = delete;
	DefaultRouteMonitor &operator=(DefaultRouteMonitor &&) = delete;

private:

	ADDRESS_FAMILY m_family;
	Callback m_callback;
	std::shared_ptr<common::logging::ILogSink> m_logSink;

	// This can't be a plain member variable.
	// We need to be able to delete it explicitly in order to have a controlled tear down.
	std::unique_ptr<common::BurstGuard> m_evaluateRoutesGuard;

	std::optional<InterfaceAndGateway> m_bestRoute;

	HANDLE m_routeNotificationHandle;
	HANDLE m_interfaceNotificationHandle;

	std::mutex m_evaluationLock;

	static void NETIOAPI_API_ RouteChangeCallback(void *context, MIB_IPFORWARD_ROW2 *row, MIB_NOTIFICATION_TYPE notificationType);
	static void NETIOAPI_API_ InterfaceChangeCallback(void *context, MIB_IPINTERFACE_ROW *row, MIB_NOTIFICATION_TYPE notificationType);

	void evaluateRoutes();
	void evaluateRoutesInner();
};

}
