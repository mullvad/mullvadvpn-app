#include "stdafx.h"
#include "nameserversource.h"

NameServerSource::NameServerSource(const std::vector<std::wstring> &ipv4NameServers,
	const std::vector<std::wstring> &ipv6NameServers)
	: m_ipv4NameServers(ipv4NameServers)
	, m_ipv6NameServers(ipv6NameServers)
{
}

void NameServerSource::setNameServers(Protocol protocol, const std::vector<std::wstring> &nameServers)
{
	{
		std::scoped_lock<std::mutex> lock(m_nameServersMutex);

		if (Protocol::IPv4 == protocol)
		{
			m_ipv4NameServers = nameServers;
		}
		else
		{
			m_ipv6NameServers = nameServers;
		}
	}

	//
	// Notify all subscribers.
	//

	std::scoped_lock<std::mutex> lock(m_subscriptionMutex);

	for (HANDLE eventHandle : m_subscriptions)
	{
		SetEvent(eventHandle);
	}
}

std::vector<std::wstring> NameServerSource::getNameServers(Protocol protocol) const
{
	std::vector<std::wstring> copy;

	std::scoped_lock<std::mutex> lock(m_nameServersMutex);

	if (Protocol::IPv4 == protocol)
	{
		copy = m_ipv4NameServers;
	}
	else
	{
		copy = m_ipv6NameServers;
	}

	return copy;
}

void NameServerSource::subscribe(HANDLE eventHandle)
{
	ResetEvent(eventHandle);

	std::scoped_lock<std::mutex> lock(m_subscriptionMutex);

	m_subscriptions.push_back(eventHandle);
}

void NameServerSource::unsubscribe(HANDLE eventHandle)
{
	std::scoped_lock<std::mutex> lock(m_subscriptionMutex);

	auto it = std::find(m_subscriptions.begin(), m_subscriptions.end(), eventHandle);

	if (m_subscriptions.end() == it)
	{
		return;
	}

	m_subscriptions.erase(it);
}
