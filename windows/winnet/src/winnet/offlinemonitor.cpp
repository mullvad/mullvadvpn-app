#include "stdafx.h"
#include "offlinemonitor.h"
#include <libcommon/error.h>
#include <libcommon/memory.h>
#include <libcommon/string.h>
#include <sstream>


using namespace std::placeholders; // for _1, _2 etc.

namespace
{

bool IsConnectedAdapter(const MIB_IF_ROW2 &iface)
{
	switch (iface.InterfaceLuid.Info.IfType)
	{
		case IF_TYPE_SOFTWARE_LOOPBACK:
		case IF_TYPE_TUNNEL:
		{
			return false;
		}
	}

	//
	// (Windows 10, and possibly others.)
	//
	// The BT adapter is erronously not marked as representing hardware.
	// By filtering on this we currently do not support BT tethering.
	//
	// Specifically, the following settings are problematic:
	//
	// InterfaceAndOperStatusFlags.HardwareInterface: 0
	// InterfaceAndOperStatusFlags.ConnectorPresent: 0
	//

	if (FALSE == iface.InterfaceAndOperStatusFlags.HardwareInterface
		&& FALSE == iface.InterfaceAndOperStatusFlags.ConnectorPresent)
	{
		return false;
	}

	//
	// Maybe checking iface.InterfaceAndOperStatusFlags.NotMediaConnected here
	// would be a good thing?
	//

	if (FALSE != iface.InterfaceAndOperStatusFlags.FilterInterface
		|| 0 == iface.PhysicalAddressLength
		|| FALSE != iface.InterfaceAndOperStatusFlags.EndPointInterface)
	{
		return false;
	}

	return
	(
		IfOperStatusUp == iface.OperStatus
		&& MediaConnectStateConnected == iface.MediaConnectState
	);
}

} // anonymous namespace


OfflineMonitor::OfflineMonitor
(
	std::shared_ptr<common::logging::ILogSink> logSink,
	Notifier notifier,
	std::shared_ptr<NetworkAdapterMonitor::IDataProvider> dataProvider
)
	: m_logSink(logSink)
	, m_notifier(notifier)
	, m_connected(false)
	, m_netAdapterMonitor(
		m_logSink,
		std::bind(&OfflineMonitor::callback, this, _1, _2, _3),
		IsConnectedAdapter,
		dataProvider
	)
{
}


OfflineMonitor::OfflineMonitor
(
	std::shared_ptr<common::logging::ILogSink> logSink,
	Notifier notifier
) : OfflineMonitor(logSink, notifier, std::make_shared<NetworkAdapterMonitor::SystemDataProvider>())
{
}


void OfflineMonitor::callback(const std::vector<MIB_IF_ROW2> &adapters, const MIB_IF_ROW2 *, NetworkAdapterMonitor::UpdateType)
{
	const auto previousConnectivity = m_connected;
	m_connected = !adapters.empty();

	if (previousConnectivity != m_connected)
	{
		m_notifier(m_connected);

		if (false == m_connected)
		{
			LogOfflineState();
		}
	}
}

void OfflineMonitor::LogOfflineState()
{
	//
	// There is a race condition here because logging is not done using the
	// same data set that the online/offline logic processes.
	//
	// Not much of a problem really, this is temporary logging.
	//

	m_logSink->info("Machine is offline");

	MIB_IF_TABLE2 *table;

	const auto status = GetIfTable2(&table);

	if (NO_ERROR != status)
	{
		m_logSink->error("Failed to acquire list of network interfaces. Aborting detailed logging");
		return;
	}

	common::memory::ScopeDestructor sd;

	sd += [table]()
	{
		FreeMibTable(table);
	};

	m_logSink->info("Begin detailed listing of network interfaces");

	for (ULONG i = 0; i < table->NumEntries; ++i)
	{
		const auto &iface = table->Table[i];

		//
		// Don't flood the log with garbage.
		//
		const auto blacklist = std::vector<std::wstring>
		{
			L"WFP Native MAC Layer LightWeight Filter",
			L"QoS Packet Scheduler",
			L"WFP 802.3 MAC Layer LightWeight Filter",
			L"Microsoft Kernel Debug Network Adapter",
			L"Software Loopback Interface",
			L"Microsoft Teredo Tunneling Adapter",
			L"Microsoft IP-HTTPS Platform Adapter",
			L"Microsoft 6to4 Adapter",
			L"WAN Miniport",
			L"WiFi Filter Driver",
			L"Microsoft Wi-Fi Direct Virtual Adapter",
		};

		bool blacklisted = false;

		for (const auto &black : blacklist)
		{
			if (nullptr != wcsstr(iface.Description, black.c_str()))
			{
				blacklisted = true;
				break;
			}
		}

		if (blacklisted)
		{
			continue;
		}

		std::stringstream ss;

		ss << "Detailed interface logging" << std::endl;
		ss << "Interface ordinal " << i << std::endl;

		{
			const auto s = std::wstring(L"  Alias: ").append(iface.Alias);
			ss << common::string::ToAnsi(s) << std::endl;
		}

		{
			const auto s = std::wstring(L"  Description: ").append(iface.Description);
			ss << common::string::ToAnsi(s) << std::endl;
		}

		ss << "  PhysicalAddressLength: " << iface.PhysicalAddressLength << std::endl;
		ss << "  Type: " << iface.Type << std::endl;
		ss << "  MediaType: " << iface.MediaType << std::endl;
		ss << "  PhysicalMediumType: " << iface.PhysicalMediumType << std::endl;
		ss << "  AccessType: " << iface.AccessType << std::endl;

		//
		// Bool cast prevents idiot stream from inserting literal 0/1.
		//

		ss << "  InterfaceAndOperStatusFlags.HardwareInterface: " << (bool)iface.InterfaceAndOperStatusFlags.HardwareInterface << std::endl;
		ss << "  InterfaceAndOperStatusFlags.FilterInterface: " << (bool)iface.InterfaceAndOperStatusFlags.FilterInterface << std::endl;
		ss << "  InterfaceAndOperStatusFlags.ConnectorPresent: " << (bool)iface.InterfaceAndOperStatusFlags.ConnectorPresent << std::endl;
		ss << "  InterfaceAndOperStatusFlags.NotAuthenticated: " << (bool)iface.InterfaceAndOperStatusFlags.NotAuthenticated << std::endl;
		ss << "  InterfaceAndOperStatusFlags.NotMediaConnected: " << (bool)iface.InterfaceAndOperStatusFlags.NotMediaConnected << std::endl;
		ss << "  InterfaceAndOperStatusFlags.Paused: " << (bool)iface.InterfaceAndOperStatusFlags.Paused << std::endl;
		ss << "  InterfaceAndOperStatusFlags.LowPower: " << (bool)iface.InterfaceAndOperStatusFlags.LowPower << std::endl;
		ss << "  InterfaceAndOperStatusFlags.EndPointInterface: " << (bool)iface.InterfaceAndOperStatusFlags.EndPointInterface << std::endl;

		ss << "  OperStatus: " << iface.OperStatus << std::endl;
		ss << "  AdminStatus: " << iface.AdminStatus << std::endl;
		ss << "  MediaConnectState: " << iface.MediaConnectState << std::endl;
		ss << "  TransmitLinkSpeed: " << iface.TransmitLinkSpeed << std::endl;

		ss << "  ReceiveLinkSpeed: " << iface.ReceiveLinkSpeed << std::endl;
		ss << "  InUcastPkts:" << iface.InUcastPkts;

		m_logSink->info(ss.str().c_str());
	}

	m_logSink->info("End detailed listing of network interfaces");
}
