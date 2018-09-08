#pragma once

#include <string>
#include <cstdint>
#include <stdexcept>

class NetSh
{
public:

	static void SetIpv4PrimaryDns(uint32_t interfaceIndex, std::wstring server);
	
	//
	// Caveat: This sets the primary DNS server if there isn't already one.
	//
	static void SetIpv4SecondaryDns(uint32_t interfaceIndex, std::wstring server);

	static void SetIpv4Dhcp(uint32_t interfaceIndex);

	static void SetIpv6PrimaryDns(uint32_t interfaceIndex, std::wstring server);
	static void SetIpv6SecondaryDns(uint32_t interfaceIndex, std::wstring server);
	static void SetIpv6Dhcp(uint32_t interfaceIndex);

private:

	NetSh();
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
