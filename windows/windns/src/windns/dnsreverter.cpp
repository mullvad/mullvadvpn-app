#include "stdafx.h"
#include "dnsreverter.h"
#include "wmi/methodcall.h"
#include <sstream>

DnsReverter::DnsReverter(std::shared_ptr<ITraceSink> traceSink)
	: m_traceSink(traceSink)
{
}

void DnsReverter::revert(wmi::IConnection &connection, const DnsConfig &config)
{
	XTRACE("Reverting DNS configuration for interface with index=", config.interfaceIndex());

	std::wstringstream ss;

	ss << L"SELECT * FROM Win32_NetworkAdapterConfiguration "
		<< L"WHERE SettingID = '" << config.id() << L"'";

	auto resultSet = connection.query(ss.str().c_str());

	if (false == resultSet.advance())
	{
		XTRACE("Unable to retrieve active configuration");
		return;
	}

	auto activeConfig = resultSet.result();
	auto targetDns = config.servers();

	nchelpers::SetDnsServers(connection, activeConfig, targetDns);
}
