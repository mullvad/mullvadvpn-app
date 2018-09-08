#include "stdafx.h"
#include "windnscontext.h"
#include "libcommon/wmi/connection.h"
#include "netconfigeventsink.h"
#include "netconfighelpers.h"

using namespace common;

WinDnsContext::WinDnsContext()
{
	m_connection = std::make_shared<wmi::Connection>(wmi::Connection::Namespace::Cimv2);
}

WinDnsContext::~WinDnsContext()
{
	try
	{
		reset();
	}
	catch (...)
	{
	}
}

void WinDnsContext::set(const std::vector<std::wstring> &servers, const ClientSinkInfo &sinkInfo)
{
	m_sinkInfo = sinkInfo;

	if (nullptr == m_notification)
	{
		m_configManager = std::make_shared<ConfigManager>(servers, this);

		//
		// Register interface configuration monitoring.
		//

		auto eventSink = std::make_shared<NetConfigEventSink>(m_connection, m_configManager, this);
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
	}
	else
	{
		ConfigManager::Mutex mutex(*m_configManager);

		m_configManager->updateServers(servers);
	}

	//
	// Discover all active interfaces and apply our DNS settings.
	//

	auto resultSet = m_connection->query(L"SELECT * from Win32_NetworkAdapterConfiguration WHERE IPEnabled = True");

	while (resultSet.advance())
	{
		nchelpers::SetDnsServers(nchelpers::GetInterfaceIndex(resultSet.result()), servers);
	}
}

void WinDnsContext::reset()
{
	if (nullptr == m_notification)
	{
		return;
	}

	m_notification->deactivate();
	m_notification = nullptr;

	//
	// Revert configs
	// Safe to do without a mutex guarding the config manager
	//

	m_configManager->processConfigs([&](const InterfaceConfig &config)
	{
		nchelpers::RevertDnsServers(config);
		return true;
	});
}

// IClientSinkProxy
void WinDnsContext::error(const char *errorMessage, const char **details, uint32_t numDetails)
{
	if (nullptr != m_sinkInfo.errorSinkInfo.sink)
	{
		m_sinkInfo.errorSinkInfo.sink(errorMessage, details, numDetails, m_sinkInfo.errorSinkInfo.context);
	}
}

// IClientSinkProxy
void WinDnsContext::config(const void *configData, uint32_t dataLength)
{
	if (nullptr != m_sinkInfo.configSinkInfo.sink)
	{
		m_sinkInfo.configSinkInfo.sink(configData, dataLength, m_sinkInfo.configSinkInfo.context);
	}
}
