#include "stdafx.h"
#include "configmanager.h"
#include "libcommon/serialization/serializer.h"
#include "libcommon/trace/xtrace.h"
#include <utility>
#include <algorithm>

ConfigManager::ConfigManager
(
	const std::vector<std::wstring> &servers,
	IClientSinkProxy *clientSinkProxy
)
	: m_servers(servers)
	, m_clientSinkProxy(clientSinkProxy)
{
}

void ConfigManager::lock()
{
	m_mutex.lock();
}

void ConfigManager::unlock()
{
	m_mutex.unlock();
}

void ConfigManager::updateServers(const std::vector<std::wstring> &servers)
{
	XTRACE(L"Updating DNS server list");
	m_servers = servers;
}

const std::vector<std::wstring> &ConfigManager::getServers() const
{
	return m_servers;
}

ConfigManager::UpdateStatus ConfigManager::updateConfig(const InterfaceConfig &previous, const InterfaceConfig &target)
{
	XTRACE(L"Interface configuration update for interface=", target.interfaceIndex());

	//
	// There are a few cases we need to deal with:
	//
	// 1/ An interface being offline and coming online.
	// 2/ An external application changing the interface settings.
	// 3/ Us changing the interface settings.
	//    a. On an interface the ConfigManager hasn't seen before.
	//    b. On an interface the ConfigManager already knows about.
	//

	const auto configIndex = target.configIndex();
	auto iter = m_configs.find(configIndex);

	if (verifyServers(target))
	{
		XTRACE(L"Update event was initiated by WINDNS or did not include DNS changes");

		//
		// If we haven't seen this config id before, it means the 'previous' instance
		// is the original configuration on the system, and as such must be recorded.
		//
		if (m_configs.end() == iter)
		{
			XTRACE(L"Creating new interface configuration entry");
			m_configs.insert(std::make_pair(configIndex, previous));

			exportConfigs();
		}

		return UpdateStatus::DnsApproved;
	}

	//
	// The update was not initiated by us so store the updated configuration.
	//
	if (m_configs.end() == iter)
	{
		XTRACE(L"Creating new interface configuration entry");
		m_configs.insert(std::make_pair(configIndex, target));
	}
	else
	{
		XTRACE(L"Updating interface configuration entry");
		iter->second.updateServers(target);
	}

	exportConfigs();

	return UpdateStatus::DnsDeviates;
}

bool ConfigManager::processConfigs(std::function<bool(const InterfaceConfig &)> configSink)
{
	for (auto it = m_configs.begin(); it != m_configs.end(); ++it)
	{
		if (false == configSink(it->second))
		{
			return false;
		}
	}

	return true;
}

bool ConfigManager::verifyServers(const InterfaceConfig &config)
{
	auto updatedServers = config.servers();

	if (nullptr == updatedServers)
	{
		return false;
	}

	return std::equal(m_servers.begin(), m_servers.end(), updatedServers->begin(), updatedServers->end());
}

void ConfigManager::exportConfigs()
{
	common::serialization::Serializer s;

	s << static_cast<uint32_t>(m_configs.size());

	for (auto it = m_configs.begin(); it != m_configs.end(); ++it)
	{
		it->second.serialize(s);
	}

	auto data = s.blob();

	m_clientSinkProxy->config(&data[0], static_cast<uint32_t>(data.size()));
}
