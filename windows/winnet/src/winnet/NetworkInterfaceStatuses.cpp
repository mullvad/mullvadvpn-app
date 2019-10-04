#include "stdafx.h"

#include "NetworkInterfaceStatuses.h"
#include <libcommon/error.h>
#include <libcommon/memory.h>
#include <sstream>

namespace
{

bool ValidInterfaceType(const MIB_IF_ROW2 &iface)
{
	switch (iface.InterfaceLuid.Info.IfType)
	{
		case IF_TYPE_SOFTWARE_LOOPBACK:
		case IF_TYPE_TUNNEL:
		{
			return false;
		}
	}

	if (FALSE != iface.InterfaceAndOperStatusFlags.FilterInterface
		|| 0 == iface.PhysicalAddressLength
		|| FALSE != iface.InterfaceAndOperStatusFlags.EndPointInterface)
	{
		return false;
	}

	return true;
}

} // anonymous namespace

NetworkInterfaceStatuses::NetworkInterfaceStatuses()
{
	MIB_IF_TABLE2 *table;

	const auto status = GetIfTable2(&table);

	THROW_UNLESS(NO_ERROR, status, "Acquire network interface table");

	common::memory::ScopeDestructor sd;

	sd += [table]()
	{
		FreeMibTable(table);
	};

	for (ULONG i = 0; i < table->NumEntries; ++i)
	{
		addInternal(table->Table[i]);
	}
}

bool NetworkInterfaceStatuses::anyConnected() const
{
	for (const auto niIter : m_cache)
	{
		const auto entry = niIter.second;

		if (entry.valid && entry.connected)
		{
			return true;
		}
	}

	return false;
}

void NetworkInterfaceStatuses::addInternal(const MIB_IF_ROW2 &iface)
{
	bool valid = ValidInterfaceType(iface);
	bool connected = valid &&
	(
		NET_IF_ADMIN_STATUS_UP == iface.AdminStatus
		&& IfOperStatusUp == iface.OperStatus
		&& MediaConnectStateConnected == iface.MediaConnectState
	);

	Entry e(
		iface.InterfaceLuid.Value,
		valid,
		connected
	);
	m_cache.insert(std::make_pair(e.luid, e));
}

void NetworkInterfaceStatuses::add(NET_LUID luid)
{
	MIB_IF_ROW2 newIface = { 0 };
	newIface.InterfaceLuid = luid;

	const auto status = GetIfEntry2(&newIface);

	if (NO_ERROR != status)
	{
		std::stringstream ss;

		ss << "GetIfEntry2() failed for LUID 0x" << std::hex << newIface.InterfaceLuid.Value
			<< " during processing of MibAddInstance, error: 0x" << status;

		throw std::runtime_error(ss.str());
	}

	//
	// The reason for removing an existing entry is that enabling
	// an interface on the adapter might change the overall properties in the
	// "row" which is merely an abstraction over all interfaces.
	//

	m_cache.erase(newIface.InterfaceLuid.Value);
	addInternal(newIface);
}

void NetworkInterfaceStatuses::remove(NET_LUID luid)
{
	m_cache.erase(luid.Value);

	MIB_IF_ROW2 newIface = { 0 };
	newIface.InterfaceLuid = luid;

	const auto status = GetIfEntry2(&newIface);

	if (NO_ERROR == status)
	{
		addInternal(newIface);
	}
}

void NetworkInterfaceStatuses::update(NET_LUID luid)
{
	MIB_IF_ROW2 newIface = { 0 };
	newIface.InterfaceLuid = luid;

	const auto status = GetIfEntry2(&newIface);

	if (NO_ERROR != status)
	{
		//
		// Only update the cache if we can look up the interface details.
		// This way, if the interface was connected and continues to be so, we don't
		// mistakenly switch the status to "offline".
		//

		std::stringstream ss;

		ss << "GetIfEntry2() failed for LUID 0x" << std::hex << newIface.InterfaceLuid.Value
			<< " during processing of MibParameterNotification, error: 0x" << status;

		throw std::runtime_error(ss.str());
	}

	m_cache.erase(newIface.InterfaceLuid.Value);
	addInternal(newIface);
}
