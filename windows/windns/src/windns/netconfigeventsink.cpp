#include "stdafx.h"
#include "netconfigeventsink.h"
#include "netconfighelpers.h"
#include "netsh.h"
#include "confineoperation.h"
#include <functional>

using namespace common;

NetConfigEventSink::NetConfigEventSink
(
	std::shared_ptr<wmi::IConnection> connection,
	std::shared_ptr<ConfigManager> configManager,
	IClientSinkProxy *clientSinkProxy
)
	: m_connection(connection)
	, m_configManager(configManager)
	, m_clientSinkProxy(clientSinkProxy)
{
}

void NetConfigEventSink::update(CComPtr<IWbemClassObject> previous, CComPtr<IWbemClassObject> target)
{
	auto forwardError = [this](const char *errorMessage, const char **details, uint32_t numDetails)
	{
		m_clientSinkProxy->error(errorMessage, details, numDetails);
	};

	ConfineOperation("Process adapter update event", forwardError, [&]()
	{
		InterfaceConfig previousConfig(previous);
		InterfaceConfig targetConfig(target);

		ConfigManager::Mutex mutex(*m_configManager);

		//
		// This is OK because the config manager will reject updates
		// that set our DNS servers.
		//
		if (ConfigManager::UpdateStatus::DnsApproved == m_configManager->updateConfig(previousConfig, targetConfig))
		{
			return;
		}

		//
		// The update was initiated from an external source.
		// Override current settings to enforce our selected DNS servers.
		//
		nchelpers::SetDnsServers(targetConfig.interfaceIndex(), m_configManager->getServers());
	});
}
