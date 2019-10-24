#include "stdafx.h"

#include "testadapterutil.h"

#include <CppUnitTest.h>

using namespace Microsoft::VisualStudio::CppUnitTestFramework;

#include <libcommon/logging/logsink.h>
#include "../../winnet/networkadaptermonitor.h"

using FilterType = NetworkAdapterMonitor::FilterType;
using UpdateSinkType = NetworkAdapterMonitor::UpdateSinkType;
using UpdateType = NetworkAdapterMonitor::UpdateType;

//
// MibIfTable
//

#include <algorithm>

void MibIfTable::add(const MIB_IF_ROW2 &row)
{
	const auto it = std::find_if(
		m_table.begin(),
		m_table.end(),
		[&row](const MIB_IF_ROW2 &elem)
		{
			return elem.InterfaceLuid.Value == row.InterfaceLuid.Value;
		}
	);

	if (m_table.end() == it)
	{
		m_table.push_back(row);
	}
	else
	{
		*it = row;
	}
}

void MibIfTable::remove(const MIB_IF_ROW2 &row)
{
	const auto it = std::find_if(
		m_table.begin(),
		m_table.end(),
		[&row](const MIB_IF_ROW2 &elem)
		{
			return elem.InterfaceLuid.Value == row.InterfaceLuid.Value;
		}
	);
	
	if (m_table.end() != it)
	{
		m_table.erase(it);
	}
}


//
// TestDataProvider
//

DWORD TestDataProvider::notifyIpInterfaceChange(
	ADDRESS_FAMILY Family,
	PIPINTERFACE_CHANGE_CALLBACK Callback,
	PVOID CallerContext,
	BOOLEAN InitialNotification,
	HANDLE *NotificationHandle
)
{
	// TODO: assert: m_callback == nullptr
	// TODO: multiple callbacks?
	m_callback = Callback;
	m_context = CallerContext;
	
	return NO_ERROR;
}

DWORD TestDataProvider::cancelMibChangeNotify2(HANDLE NotificationHandle)
{
	// TODO: assert: m_callback != nullptr
	// TODO: multiple callbacks?
	m_callback = nullptr;
	m_context = nullptr;
	
	return NO_ERROR;
}

DWORD TestDataProvider::getIfTable2(PMIB_IF_TABLE2 *tableOut)
{
	MIB_IF_TABLE2 *tableCopy = reinterpret_cast<MIB_IF_TABLE2*>(new uint8_t[
		sizeof(MIB_IF_TABLE2)
		+ sizeof(MIB_IF_ROW2) * m_adapterTable.entries().size()
	]);

	tableCopy->NumEntries = static_cast<ULONG>(m_adapterTable.entries().size());

	std::copy(
		m_adapterTable.entries().begin(),
		m_adapterTable.entries().end(),
		static_cast<MIB_IF_ROW2*>(tableCopy->Table)
	);

	*tableOut = tableCopy;
	return NO_ERROR;
}

void TestDataProvider::freeMibTable(PVOID Memory)
{
	delete[] reinterpret_cast<uint8_t*>(Memory);
}

DWORD TestDataProvider::getIfEntry2(PMIB_IF_ROW2 Row)
{
	// TODO: accept InterfaceIndex as well
	// FIXME: should ERROR_INVALID_PARAMETER be returned if LUID = 0?

	if (Row == nullptr)
	{
		return ERROR_INVALID_PARAMETER;
	}
	
	const auto it = std::find_if(
		m_adapterTable.entries().begin(),
		m_adapterTable.entries().end(),
		[&Row](const MIB_IF_ROW2 &entry)
		{
			return entry.InterfaceLuid.Value == Row->InterfaceLuid.Value;
		}
	);

	if (m_adapterTable.entries().end() == it)
	{
		return ERROR_FILE_NOT_FOUND;
	}

	*Row = *it;
	
	return NO_ERROR;
}

DWORD TestDataProvider::getIpInterfaceEntry(PMIB_IPINTERFACE_ROW Row)
{
	// TODO: accept InterfaceIndex as well
	// FIXME: should ERROR_INVALID_PARAMETER be returned if LUID = 0?
	
	if (Row == nullptr)
	{
		return ERROR_INVALID_PARAMETER;
	}
	if (Row->Family != AF_INET && Row->Family != AF_INET6)
	{
		return ERROR_INVALID_PARAMETER;
	}

	bool foundMatchingLuid = false;

	for (auto it = m_ipInterfaces.begin(); m_ipInterfaces.end() != it; ++it)
	{
		if (it->InterfaceLuid.Value != Row->InterfaceLuid.Value)
		{
			continue;
		}

		foundMatchingLuid = true;

		if (Row->Family == it->Family)
		{
			*Row = *it;
			return NO_ERROR;
		}
	}

	if (foundMatchingLuid)
	{
		// "the Row parameter does not match the IP address family
		// specified in the Family member in the
		// MIB_IPINTERFACE_ROW structure."
		return ERROR_NOT_FOUND;
	}

	//
	// The LUID is also valid if it exists among adapters
	// without an IP interface
	//
	const auto it = std::find_if(
		m_adapterTable.entries().begin(),
		m_adapterTable.entries().end(),
		[Row](const MIB_IF_ROW2 &elem)
		{
			return Row->InterfaceLuid.Value == elem.InterfaceLuid.Value;
		}
	);

	if (m_adapterTable.entries().end() != it)
	{
		return ERROR_NOT_FOUND;
	}
	
	return ERROR_FILE_NOT_FOUND;
}

void TestDataProvider::addAdapter(const MIB_IF_ROW2& adapter)
{
	m_adapterTable.add(adapter);
}

void TestDataProvider::addIpInterface(const MIB_IF_ROW2& adapter, const MIB_IPINTERFACE_ROW& iface)
{
	addAdapter(adapter);
	m_ipInterfaces.push_back(iface);
}

void TestDataProvider::removeAdapter(const MIB_IF_ROW2& adapter)
{
	for (auto it = m_ipInterfaces.begin(); m_ipInterfaces.end() != it; )
	{
		if (it->InterfaceLuid.Value == adapter.InterfaceLuid.Value)
		{
			it = m_ipInterfaces.erase(it);
		}
		else
		{
			++it;
		}
	}

	m_adapterTable.remove(adapter);
}

void TestDataProvider::removeIpInterface(const MIB_IPINTERFACE_ROW& iface)
{
	const auto it = std::find_if(
		m_ipInterfaces.begin(),
		m_ipInterfaces.end(),
		[&iface](const MIB_IPINTERFACE_ROW &elem)
		{
			return iface.InterfaceLuid.Value == elem.InterfaceLuid.Value
				&& iface.Family == elem.Family;
		}
	);

	if (m_ipInterfaces.end() != it)
	{
		m_ipInterfaces.erase(it);
	}
}

void TestDataProvider::sendEvent(MIB_IPINTERFACE_ROW *hint, MIB_NOTIFICATION_TYPE updateType)
{
	m_callback(m_context, hint, updateType);
}
