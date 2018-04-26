#pragma once

#include "dnsconfig.h"
#include "itracesink.h"
#include <map>
#include <string>
#include <mutex>
#include <memory>
#include <functional>

class ConfigManager
{
public:

	struct Mutex
	{
		Mutex(const Mutex &) = delete;
		Mutex &operator=(const Mutex &) = delete;
		Mutex(Mutex &&) = delete;
		Mutex &operator=(Mutex &&) = delete;

		Mutex(ConfigManager &manager)
			: m_manager(manager)
		{
			m_manager.lock();
		}

		~Mutex()
		{
			m_manager.unlock();
		}

		ConfigManager &m_manager;
	};

	//
	// "servers" specifies the set of servers used when overriding settings.
	// This enables filtering out the corresponding event.
	//
	ConfigManager
	(
		const std::vector<std::wstring> &servers,
		std::shared_ptr<ITraceSink> traceSink = std::make_shared<NullTraceSink>()
	);

	void lock();
	void unlock();

	void updateServers(const std::vector<std::wstring> &servers);
	const std::vector<std::wstring> &getServers() const;

	bool updateConfig(DnsConfig &&config);

	bool processConfigs(std::function<bool(const DnsConfig &)> configSink);

private:

	std::mutex m_mutex;
	std::vector<std::wstring> m_servers;
	std::map<std::wstring, DnsConfig> m_configs;

	std::shared_ptr<ITraceSink> m_traceSink;

	bool validUpdate(const DnsConfig &config);
};
