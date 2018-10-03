#pragma once

#include "windns.h"
#include "clientsinkinfo.h"
#include "ilogsink.h"
#include "recoverysink.h"
#include "nameserversource.h"
#include "dnsagent.h"
#include <vector>
#include <string>
#include <memory>

class WinDnsContext
{
public:

	WinDnsContext(ILogSink *logSink);
	~WinDnsContext();

	void set(const std::vector<std::wstring> &ipv4NameServers, const std::vector<std::wstring> &ipv6NameServers,
		const RecoverySinkInfo &recoverySinkInfo);

	void reset();

private:

	ILogSink *m_logSink;

	std::unique_ptr<RecoverySink> m_recoverySink;
	std::unique_ptr<NameServerSource> m_nameServerSource;

	std::unique_ptr<DnsAgent> m_ipv4Agent;
	std::unique_ptr<DnsAgent> m_ipv6Agent;
};
