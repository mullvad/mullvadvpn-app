#pragma once

#include <libcommon/logging/ilogsink.h>
#include <map>
#include <winsock2.h>
#include <ws2ipdef.h>
#include <iphlpapi.h>
#include <windows.h>

class NetworkInterfaceStatuses
{
public:

	struct Entry
	{
		// Last known state.
		bool connected;

		// Unique interface identifier.
		uint64_t luid;

		// Whether this is a physical adapter or not.
		bool valid;

		Entry(uint64_t luid, bool valid, bool connected) :
			luid(luid), valid(valid), connected(connected)
		{
		}

		Entry() :
			connected(false), luid(0), valid(false)
		{
		}
	};

	NetworkInterfaceStatuses();

	void Add(NET_LUID luid);
	void Remove(NET_LUID luid);
	void Update(NET_LUID luid);
	bool AnyConnected();

private:

	void AddInternal(const MIB_IF_ROW2 &iface);
	std::map<uint64_t, Entry> m_cache;
};
