#include "stdafx.h"
#include "netconfigeventsink.h"
#include "windns/netconfighelpers.h"

NetConfigEventSink::NetConfigEventSink(std::shared_ptr<wmi::IConnection> connection, std::shared_ptr<ConfigManager> configManager)
	: m_connection(connection)
	, m_configManager(configManager)
{
}

void NetConfigEventSink::update(CComPtr<IWbemClassObject> previous, CComPtr<IWbemClassObject> target)
{
	InterfaceConfig previousConfig(previous);
	InterfaceConfig targetConfig(target);

	ConfigManager::Mutex mutex(*m_configManager);

	//
	// This is OK because the config manager will reject updates
	// that set our DNS servers.
	//
	if (ConfigManager::UpdateType::WinDnsEnforced == m_configManager->updateConfig(previousConfig, targetConfig))
	{
		return;
	}

	//
	// The update was initiated from an external source.
	// Override current settings to enforce our selected DNS servers.
	//
	nchelpers::SetDnsServers(targetConfig.interfaceIndex(), m_configManager->getServers());
}
