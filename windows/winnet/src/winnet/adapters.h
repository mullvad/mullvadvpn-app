#pragma once

#include <vector>
#include <winsock2.h>
#include <windows.h>
#include <iphlpapi.h>

//
// This is a thin wrapper on top of GetAdaptersAddresses()
// in order to simplify memory management.
//

class Adapters
{
	std::vector<uint8_t> m_buffer;
	mutable const IP_ADAPTER_ADDRESSES *m_currentEntry;

public:

	Adapters(const Adapters &) = delete;
	Adapters &operator=(const Adapters &) = delete;

	Adapters(Adapters &&rhs)
		: m_buffer(std::move(rhs.m_buffer))
		, m_currentEntry(rhs.m_currentEntry)
	{
	}

	Adapters(DWORD family, DWORD flags);

	const IP_ADAPTER_ADDRESSES *next() const;

	void reset() const
	{
		if (false == m_buffer.empty())
		{
			m_currentEntry = reinterpret_cast<const IP_ADAPTER_ADDRESSES *>(&m_buffer[0]);
		}
	}
};
