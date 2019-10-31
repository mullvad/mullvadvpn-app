#include "stdafx.h"
#include <libcommon/error.h>
#include "defaultroutemonitor.h"
#include "helpers.h"

namespace winnet::routing
{

namespace
{

const uint32_t POINT_TWO_SECOND_BURST = 200;
const uint32_t TWO_SECOND_INTERFERENCE = 2000;

} // anonymous namespace

DefaultRouteMonitor::DefaultRouteMonitor
(
	ADDRESS_FAMILY family,
	Callback callback,
	std::shared_ptr<common::logging::ILogSink> logSink
)
	: m_family(family)
	, m_callback(callback)
	, m_logSink(logSink)
	, m_evaluateRoutesGuard(std::bind(&DefaultRouteMonitor::evaluateRoutes, this),
		POINT_TWO_SECOND_BURST, TWO_SECOND_INTERFERENCE)
{
	try
	{
		m_bestRoute = GetBestDefaultRoute(m_family);
	}
	catch (...)
	{
	}

	const auto status = NotifyRouteChange2(AF_UNSPEC, RouteChangeCallback, this, FALSE, &m_routeNotificationHandle);

	THROW_UNLESS(NO_ERROR, status, "Register for route table change notifications");

	try
	{
		const auto s2 = NotifyIpInterfaceChange(AF_UNSPEC, InterfaceChangeCallback, this,
			FALSE, &m_interfaceNotificationHandle);

		THROW_UNLESS(NO_ERROR, status, "Register for network interface change notifications");
	}
	catch (...)
	{
		CancelMibChangeNotify2(m_routeNotificationHandle);
		throw;
	}
}

DefaultRouteMonitor::~DefaultRouteMonitor()
{
	CancelMibChangeNotify2(m_interfaceNotificationHandle);
	CancelMibChangeNotify2(m_routeNotificationHandle);
}

//static
void NETIOAPI_API_ DefaultRouteMonitor::RouteChangeCallback
(
	void *context,
	MIB_IPFORWARD_ROW2 *row,
	MIB_NOTIFICATION_TYPE
)
{
	//
	// We're only interested in changes that add/remove/update a default route.
	//

	if (0 != row->DestinationPrefix.PrefixLength
		|| false == RouteHasGateway(*row))
	{
		return;
	}

	reinterpret_cast<DefaultRouteMonitor *>(context)->m_evaluateRoutesGuard.trigger();
}

//static
void NETIOAPI_API_ DefaultRouteMonitor::InterfaceChangeCallback
(
	void *context,
	MIB_IPINTERFACE_ROW *,
	MIB_NOTIFICATION_TYPE
)
{
	reinterpret_cast<DefaultRouteMonitor *>(context)->m_evaluateRoutesGuard.trigger();
}

void DefaultRouteMonitor::evaluateRoutes()
{
	try
	{
		evaluateRoutesInner();
	}
	catch (const std::exception &ex)
	{
		const auto msg = std::string("Failure while evaluating route table: ").append(ex.what());
		m_logSink->error(msg.c_str());
	}
	catch (...)
	{
		m_logSink->error("Unspecified failure while evaluating route table");
	}
}

void DefaultRouteMonitor::evaluateRoutesInner()
{
	std::optional<InterfaceAndGateway> currentBestRoute;

	try
	{
		currentBestRoute = GetBestDefaultRoute(m_family);
	}
	catch (...)
	{
	}

	//
	// If there was no default route previously.
	//

	if (false == m_bestRoute.has_value())
	{
		if (currentBestRoute.has_value())
		{
			m_bestRoute = currentBestRoute;
			m_callback(EventType::Updated, m_bestRoute);
		}

		return;
	}

	//
	// There used to be a default route.
	// If there is not currently a default route.
	//

	if (false == currentBestRoute.has_value())
	{
		m_bestRoute.reset();
		m_callback(EventType::Removed, std::nullopt);

		return;
	}

	//
	// The current best route may have changed.
	//

	if (m_bestRoute.value() != currentBestRoute.value())
	{
		m_bestRoute = currentBestRoute;
		m_callback(EventType::Updated, m_bestRoute);
	}
}

}
