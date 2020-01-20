#pragma once

#include <libcommon/logging/ilogsink.h>
#include <libcommon/process/applicationrunner.h>
#include <string>
#include <vector>
#include <cstdint>
#include <stdexcept>
#include <memory>

class NetSh
{
public:

	NetSh(std::shared_ptr<common::logging::ILogSink> logSink);

	void setIpv4StaticDns(uint32_t interfaceIndex,
		const std::vector<std::wstring> &nameServers, uint32_t timeout = 0);

	void setIpv4DhcpDns(uint32_t interfaceIndex, uint32_t timeout = 0);

	void setIpv6StaticDns(uint32_t interfaceIndex,
		const std::vector<std::wstring> &nameServers, uint32_t timeout = 0);

	void setIpv6DhcpDns(uint32_t interfaceIndex, uint32_t timeout = 0);

private:

	std::shared_ptr<common::logging::ILogSink> m_logSink;
	std::wstring m_netShPath;

	void validateShellOut(common::process::ApplicationRunner &netsh, uint32_t timeout);
};
