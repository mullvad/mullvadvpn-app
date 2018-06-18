#pragma once

#include "interfaceconfig.h"
#include "clientsinkinfo.h"
#include <map>
#include <string>
#include <mutex>
#include <memory>
#include <functional>

//
// The ConfigManager is engineered to track the "real" DNS configuration for an adapter.
//
// The situation is somewhat complicated, because a given system may have several adapters, which
// in turn may have several configurations?
//
// Every update for every configuration is recorded, bar the ones that correspond to us
// overriding the DNS settings.
//

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
		const ConfigSinkInfo &configSinkInfo
	);

	//
	// The ConfigManager is shared between threads.
	// Locking is managed externally for reasons of efficiency.
	//
	void lock();
	void unlock();

	//
	// Notify the ConfigManager that servers used when overriding DNS settings have changed.
	//
	void updateServers(const std::vector<std::wstring> &servers);

	//
	// Update the callback used for persisting settings.
	//
	void updateConfigSink(const ConfigSinkInfo &configSinkInfo);

	//
	// Get the current set of servers used for overriding DNS settings.
	//
	const std::vector<std::wstring> &getServers() const;

	//
	// Notify the ConfigManager that a live configuration has been updated.
	//
	enum class UpdateStatus
	{
		DnsApproved,
		DnsDeviates
	};

	UpdateStatus updateConfig(const InterfaceConfig &previous, const InterfaceConfig &target);

	//
	// Enumerate recorded configs.
	//
	bool processConfigs(std::function<bool(const InterfaceConfig &)> configSink);

private:

	std::mutex m_mutex;

	std::vector<std::wstring> m_servers;
	ConfigSinkInfo m_configSinkInfo;

	//
	// Organize configs based on their system assigned index.
	//
	std::map<uint32_t, InterfaceConfig> m_configs;

	//
	// Check DNS server list to see if it matches what we're trying to enforce.
	//
	bool verifyServers(const InterfaceConfig &config);

	//
	// Bundle the current config details and send them into the config sink.
	//
	void exportConfigs();
};
