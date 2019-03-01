#include "stdafx.h"
#include "windnscontext.h"
#include "confineoperation.h"
#include "recoveryformatter.h"
#include "recoverylogic.h"
#include <functional>

using namespace common;

WinDnsContext::WinDnsContext(ILogSink *logSink)
	: m_logSink(logSink)
{
	if (nullptr == logSink)
	{
		throw std::runtime_error("Invalid logger sink");
	}
}

WinDnsContext::~WinDnsContext()
{
	ConfineOperation("Reset DNS settings", m_logSink, [this]()
	{
		this->reset();
	});
}

void WinDnsContext::set(const std::vector<std::wstring> &ipv4NameServers,
	const std::vector<std::wstring> &ipv6NameServers, const RecoverySinkInfo &recoverySinkInfo)
{
	//
	// The 'sink' and 'source' instances must be kept alive for the lifetime of the agents.
	//

	if (!m_recoverySink)
	{
		m_recoverySink = std::make_unique<RecoverySink>(recoverySinkInfo);
	}
	else
	{
		m_recoverySink->setTarget(recoverySinkInfo);
	}

    // Clearing DNS agents if the new server lists are empty, as they aren't needed.
    if (ipv4NameServers.empty())
    {
        m_ipv4Agent.reset();
    }
    if (ipv6NameServers.empty())
    {
        m_ipv6Agent.reset();
    }

	if (!m_nameServerSource)
	{
		m_nameServerSource = std::make_unique<NameServerSource>(ipv4NameServers, ipv6NameServers);
	}
	else
	{
		m_nameServerSource->setNameServers(Protocol::IPv4, ipv4NameServers);
		m_nameServerSource->setNameServers(Protocol::IPv6, ipv6NameServers);
	}

	//
	// Instantiate agents unless they're already set up or the relevant server lists are empty
	//
	if (!m_ipv4Agent && !ipv4NameServers.empty())
	{
		m_ipv4Agent = std::make_unique<DnsAgent>(Protocol::IPv4, m_nameServerSource.get(), m_recoverySink.get(), m_logSink);
    }
 

	if (!m_ipv6Agent && !ipv6NameServers.empty())
	{
		m_ipv6Agent = std::make_unique<DnsAgent>(Protocol::IPv6, m_nameServerSource.get(), m_recoverySink.get(), m_logSink);
	}
}

void WinDnsContext::reset()
{
	if (!m_ipv4Agent && !m_ipv6Agent)
	{
		return;
	}

	//
	// Destructing the agents will abort all monitoring + enforcing.
	//

	if (m_ipv4Agent)
	{
		m_ipv4Agent.reset(nullptr);
	}

	if (m_ipv6Agent)
	{
		m_ipv6Agent.reset(nullptr);
	}

	auto recoveryData = RecoveryFormatter::Unpack(m_recoverySink->recoveryData());

	RecoveryLogic::RestoreInterfaces(recoveryData, m_logSink);
}
