#pragma once

#include "inameserversource.h"
#include <mutex>

class NameServerSource : public INameServerSource
{
public:

	NameServerSource(const std::vector<std::wstring> &ipv4NameServers,
		const std::vector<std::wstring> &ipv6NameServers);

	void setNameServers(Protocol protocol, const std::vector<std::wstring> &nameServers);

	std::vector<std::wstring> getNameServers(Protocol protocol) const override;

	void subscribe(HANDLE eventHandle) override;
	void unsubscribe(HANDLE eventHandle) override;

private:

	mutable std::mutex m_nameServersMutex;

	std::vector<std::wstring> m_ipv4NameServers;
	std::vector<std::wstring> m_ipv6NameServers;

	std::mutex m_subscriptionMutex;
	std::list<HANDLE> m_subscriptions;
};
