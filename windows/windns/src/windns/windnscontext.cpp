#include "stdafx.h"
#include "windnscontext.h"
#include "wmi/connection.h"
#include "netconfigeventsink.h"
#include "dnsreverter.h"

WinDnsContext::WinDnsContext()
{
	m_connection = std::make_shared<wmi::Connection>(wmi::Connection::Namespace::Cimv2);
}

bool WinDnsContext::set(const std::vector<std::wstring> &servers, const ClientSinkInfo &sinkInfo)
{
	m_sinkInfo = sinkInfo;

	m_configManager = std::make_shared<ConfigManager>(servers);

	//
	// Register interface configuration monitoring.
	//

	auto eventSink = std::make_shared<NetConfigEventSink>(m_connection, m_configManager);
	auto eventDispatcher = CComPtr<wmi::IEventDispatcher>(new wmi::ModificationEventDispatcher(eventSink));

	m_notification = std::make_unique<wmi::Notification>(m_connection, eventDispatcher);

	m_notification->activate
	(
		L"SELECT * "
		L"FROM __InstanceModificationEvent "
		L"WITHIN 1 "
		L"WHERE TargetInstance ISA 'Win32_NetworkAdapterConfiguration'"
		L"AND TargetInstance.IPEnabled = True"
	);

	//
	// Discover all active interfaces and apply our DNS settings.
	//

	auto resultSet = m_connection->query(L"SELECT * from Win32_NetworkAdapterConfiguration WHERE IPEnabled = True");

	while (resultSet.advance())
	{
		nchelpers::SetDnsServers(*m_connection, resultSet.result(), &servers);
	}

	return true;
}

bool WinDnsContext::reset()
{
	if (nullptr == m_notification)
	{
		return true;
	}

	m_notification->deactivate();

	//
	// Revert configs
	// Safe to do without a mutex guarding the config manager
	//

	DnsReverter dnsReverter;

	m_configManager->processConfigs([&](const DnsConfig &config)
	{
		dnsReverter.revert(*m_connection, config);

		return true;
	});

	return true;
}
