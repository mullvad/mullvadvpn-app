#include "stdafx.h"
#include "configmanager.h"
#include <utility>
#include <algorithm>

ConfigManager::ConfigManager
(
	const std::vector<std::wstring> &servers,
	std::shared_ptr<ITraceSink> traceSink
)
	: m_servers(servers)
	, m_traceSink(traceSink)
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

bool ConfigManager::updateConfig(DnsConfig &&config)
{
	XTRACE(L"Interface configuration update for interface =", config.interfaceIndex());
	
	if (false == validUpdate(config))
	{
		XTRACE(L"Ignoring interface configuration update");
		return false;
	}

	auto iter = m_configs.find(config.id());

	if (m_configs.end() == iter)
	{
		XTRACE(L"Creating new interface configuration entry");
		m_configs.insert(std::make_pair(config.id(), std::move(config)));
	}
	else
	{
		XTRACE(L"Updating interface configuration entry");
		iter->second = std::move(config);
	}

	return true;
}

bool ConfigManager::processConfigs(std::function<bool(const DnsConfig &)> configSink)
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

bool ConfigManager::validUpdate(const DnsConfig &config)
{
	auto updatedServers = config.servers();

	if (nullptr == updatedServers)
	{
		return true;
	}

	return false == std::equal(m_servers.begin(), m_servers.end(), updatedServers->begin(), updatedServers->end());
}
