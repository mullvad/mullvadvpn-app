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
	, m_refreshCurrentRoute(false)
	, m_evaluateRoutesGuard(std::make_unique<common::BurstGuard>(
		std::bind(&DefaultRouteMonitor::evaluateRoutes, this),
		POINT_TWO_SECOND_BURST,
		TWO_SECOND_INTERFERENCE
	))
{
	std::scoped_lock<std::mutex> lock(m_evaluationLock);

	auto status = NotifyRouteChange2(AF_UNSPEC, RouteChangeCallback, this, FALSE, &m_routeNotificationHandle);

	if (NO_ERROR != status)
	{
		THROW_WINDOWS_ERROR(status, "Register for route table change notifications");
	}

	status = NotifyIpInterfaceChange(AF_UNSPEC, InterfaceChangeCallback, this,
		FALSE, &m_interfaceNotificationHandle);

	if (NO_ERROR != status)
	{
		CancelMibChangeNotify2(m_routeNotificationHandle);
		THROW_WINDOWS_ERROR(status, "Register for network interface change notifications");
	}

	status = NotifyUnicastIpAddressChange(AF_UNSPEC, AddressChangeCallback, this,
		FALSE, &m_addressNotificationHandle);

	if (NO_ERROR != status)
	{
		CancelMibChangeNotify2(m_routeNotificationHandle);
		CancelMibChangeNotify2(m_interfaceNotificationHandle);
		THROW_WINDOWS_ERROR(status, "Register for unicast address change notifications");
	}

	try
	{
		m_bestRoute = GetBestDefaultRoute(m_family);
	}
	catch (...)
	{
	}
}

DefaultRouteMonitor::~DefaultRouteMonitor()
{
	//
	// Cancel notifications to stop triggering the BurstGuard.
	//

	CancelMibChangeNotify2(m_addressNotificationHandle);
	CancelMibChangeNotify2(m_interfaceNotificationHandle);
	CancelMibChangeNotify2(m_routeNotificationHandle);

	//
	// Controlled destruction of BurstGuard to prevent it from calling here
	// after other member variables have been destructed.
	//

	m_evaluateRoutesGuard.reset();
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

	const auto monitor = reinterpret_cast<DefaultRouteMonitor*>(context);
	monitor->updateRefreshFlag(row->InterfaceLuid, row->InterfaceIndex);
	monitor->m_evaluateRoutesGuard->trigger();
}

//static
void NETIOAPI_API_ DefaultRouteMonitor::InterfaceChangeCallback
(
	void *context,
	MIB_IPINTERFACE_ROW *row,
	MIB_NOTIFICATION_TYPE
)
{
	const auto monitor = reinterpret_cast<DefaultRouteMonitor*>(context);
	monitor->updateRefreshFlag(row->InterfaceLuid, row->InterfaceIndex);
	monitor->m_evaluateRoutesGuard->trigger();
}

//static
void NETIOAPI_API_ DefaultRouteMonitor::AddressChangeCallback
(
	void *context,
	MIB_UNICASTIPADDRESS_ROW *row,
	MIB_NOTIFICATION_TYPE
)
{
	const auto monitor = reinterpret_cast<DefaultRouteMonitor*>(context);
	monitor->updateRefreshFlag(row->InterfaceLuid, row->InterfaceIndex);
	monitor->m_evaluateRoutesGuard->trigger();
}

void DefaultRouteMonitor::updateRefreshFlag(const NET_LUID &luid, const NET_IFINDEX &index)
{
	std::scoped_lock<std::mutex> lock(m_evaluationLock);

	if (!m_bestRoute.has_value())
	{
		return;
	}

	if (luid.Value == m_bestRoute->iface.Value)
	{
		m_refreshCurrentRoute = true;
		return;
	}

	if (luid.Value != 0)
	{
		return;
	}

	NET_IFINDEX defaultInterfaceIndex = 0;
	const auto routeLuid = &m_bestRoute->iface;
	ConvertInterfaceLuidToIndex(routeLuid, &defaultInterfaceIndex);
	m_refreshCurrentRoute = index == defaultInterfaceIndex ||
		(defaultInterfaceIndex == NET_IFINDEX_UNSPECIFIED);
}

void DefaultRouteMonitor::evaluateRoutes()
{
	std::scoped_lock<std::mutex> lock(m_evaluationLock);

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

	bool refreshCurrent = m_refreshCurrentRoute;
	m_refreshCurrentRoute = false;

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

		return;
	}

	//
	// Interface details may have changed.
	//

	if (refreshCurrent)
	{
		m_callback(EventType::UpdatedDetails, m_bestRoute);
	}
}

}
