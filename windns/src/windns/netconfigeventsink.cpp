#include "stdafx.h"
#include "netconfigeventsink.h"
#include "windns/netconfighelpers.h"

NetConfigEventSink::NetConfigEventSink(std::shared_ptr<wmi::IConnection> connection, std::shared_ptr<ConfigManager> configManager)
	: m_connection(connection)
	, m_configManager(configManager)
{
}

void NetConfigEventSink::update(CComPtr<IWbemClassObject> instance)
{
	DnsConfig config(instance);

	ConfigManager::Mutex mutex(*m_configManager);

	//
	// This is OK because the config manager will reject updates
	// that set our DNS servers.
	//
	auto updated = m_configManager->updateConfig(std::move(config));

	if (updated)
	{
		//
		// Override current settings to use our DNS servers.
		//
		auto servers = m_configManager->getServers();
		nchelpers::SetDnsServers(*m_connection, instance, &servers);
	}
}
