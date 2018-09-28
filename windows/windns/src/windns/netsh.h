#pragma once

#include "ilogsink.h"
#include <libcommon/applicationrunner.h>
#include <string>
#include <vector>
#include <cstdint>
#include <stdexcept>

class NetSh
{
public:

	static void Construct(ILogSink *logSink);

	static NetSh &Instance();

	void SetIpv4StaticDns(uint32_t interfaceIndex,
		const std::vector<std::wstring> &nameServers, uint32_t timeout = 0);

	void SetIpv4DhcpDns(uint32_t interfaceIndex, uint32_t timeout = 0);

	void SetIpv6StaticDns(uint32_t interfaceIndex,
		const std::vector<std::wstring> &nameServers, uint32_t timeout = 0);

	void SetIpv6DhcpDns(uint32_t interfaceIndex, uint32_t timeout = 0);

	static uint32_t ConvertInterfaceGuidToIndex(const std::wstring &interfaceGuid);

private:

	ILogSink *m_logSink;
	std::wstring m_netShPath;

	NetSh(ILogSink *logSink);

	void ValidateShellOut(common::ApplicationRunner &netsh, uint32_t timeout);
};

class NetShError : public std::exception
{
public:

	NetShError(std::string &&error, std::vector<std::string> &&details)
		: std::exception(error.c_str())
		, m_error(std::move(error))
		, m_details(std::move(details))
	{
	}

	const std::vector<std::string> &details()
	{
		return m_details;
	}

private:

	const std::string m_error;
	const std::vector<std::string> m_details;
};
