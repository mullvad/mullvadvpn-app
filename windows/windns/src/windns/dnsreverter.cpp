#include "stdafx.h"
#include "dnsreverter.h"
#include "netsh.h"

DnsReverter::DnsReverter(std::shared_ptr<ITraceSink> traceSink)
	: m_traceSink(traceSink)
{
}

void DnsReverter::revert(const InterfaceConfig &config)
{
	XTRACE("Reverting DNS configuration for interface with index=", config.interfaceIndex());

	auto servers = config.servers();

	if (config.dhcp() || nullptr == servers || 0 == servers->size())
	{
		NetSh::SetIpv4Dhcp(config.interfaceIndex());
		return;
	}

	NetSh::SetIpv4PrimaryDns(config.interfaceIndex(), (*servers)[0]);

	if (servers->size() > 1)
	{
		NetSh::SetIpv4SecondaryDns(config.interfaceIndex(), (*servers)[1]);
	}
}
