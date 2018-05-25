#include "stdafx.h"
#include "windnscontext.h"
#include "wmi/connection.h"
#include "netconfigeventsink.h"
#include "dnsreverter.h"

WinDnsContext::WinDnsContext()
{
	m_connection = std::make_shared<wmi::Connection>(wmi::Connection::Namespace::Cimv2);
}

bool WinDnsContext::set(const std::vector<std::wstring> &servers, WinDnsErrorSink /*errorSink*/, void * /*errorContext*/)
{
	m_configManager = std::make_shared<ConfigManager>(servers);

	//
	// See test app for details.
	//
	// Discover all active interface configurations.
	//

	auto resultSet = m_connection->query(L"SELECT * from Win32_NetworkAdapterConfiguration WHERE IPEnabled = True");

	while (resultSet.advance())
	{
		auto config = DnsConfig(resultSet.result());
		m_configManager->updateConfig(std::move(config));
	}

	//
	// Register interface configuration monitoring.
	//

	auto eventSink = std::make_shared<NetConfigEventSink>(m_connection, m_configManager);
	auto eventSinkWrapper = CComPtr<wmi::EventSink>(new wmi::EventSink(eventSink));

	m_notification = std::make_unique<wmi::Notification>(m_connection, eventSinkWrapper);

	m_notification->activate
	(
		L"SELECT * "
		L"FROM __InstanceModificationEvent "
		L"WITHIN 1 "
		L"WHERE TargetInstance ISA 'Win32_NetworkAdapterConfiguration'"
		L"AND TargetInstance.IPEnabled = True"
	);

	//
	// Apply our DNS settings
	//

	{
		ConfigManager::Mutex mutex(*m_configManager);

		m_configManager->processConfigs([&](const DnsConfig &config)
		{
			std::wstringstream ss;

			ss << L"SELECT * FROM Win32_NetworkAdapterConfiguration "
				<< L"WHERE SettingID = '" << config.id() << L"'";

			auto resultSet = m_connection->query(ss.str().c_str());

			if (resultSet.advance())
			{
				auto activeConfig = resultSet.result();
				nchelpers::SetDnsServers(*m_connection, activeConfig, &servers);
			}

			// Continue with the next interface configuration.
			return true;
		});
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
