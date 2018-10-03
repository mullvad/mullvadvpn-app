#include "stdafx.h"

#include <libcommon/registry/registry.h>

#include <vector>
#include <string>
#include <iostream>
#include <algorithm>

// Use source files directly from windns.
#include "../../windns/interfacesnap.h"
#include "../../windns/registrypaths.h"
#include "../../windns/types.h"

struct InterfaceData
{
	InterfaceData(const std::wstring &_interfaceGuid, InterfaceSnap _snap)
		: interfaceGuid(_interfaceGuid)
		, snap(_snap)
	{
	}

	std::wstring interfaceGuid;
	InterfaceSnap snap;
};

std::vector<std::wstring> DiscoverInterfaces(Protocol protocol)
{
	auto regKey = common::registry::Registry::OpenKey(HKEY_LOCAL_MACHINE, RegistryPaths::InterfaceRoot(protocol));

	std::vector<std::wstring> interfaces;

	interfaces.reserve(20);

	regKey->enumerateSubKeys([&interfaces](const std::wstring &keyName)
	{
		interfaces.push_back(keyName);
		return true;
	});

	return interfaces;
}

std::vector<InterfaceData> CreateInterfaceRecords(Protocol protocol, const std::vector<std::wstring> &interfaces)
{
	std::vector<InterfaceData> records;

	records.reserve(interfaces.size());

	for (const auto &iface : interfaces)
	{
		records.emplace_back(iface, InterfaceSnap(protocol, iface));
	}

	return records;
}

void CreateSnapshots(std::vector<InterfaceData> &v4, std::vector<InterfaceData> &v6)
{
	v4 = CreateInterfaceRecords(Protocol::IPv4, DiscoverInterfaces(Protocol::IPv4));
	v6 = CreateInterfaceRecords(Protocol::IPv6, DiscoverInterfaces(Protocol::IPv6));
}

void VerifyProtocolSnapshots(const std::vector<InterfaceData> &first, const std::vector<InterfaceData> &second)
{
	for (const auto &firstRecord : first)
	{
		const auto interfaceGuid = firstRecord.interfaceGuid;

		auto secondRecord = std::find_if(second.begin(), second.end(), [&interfaceGuid](const InterfaceData &candidate)
		{
			return interfaceGuid == candidate.interfaceGuid;
		});

		if (second.end() == secondRecord)
		{
			std::wcout << L"Interface " << interfaceGuid << L" has been removed from the system" << std::endl;
			continue;
		}

		const auto serversBefore = firstRecord.snap.nameServers();
		const auto serversAfter = secondRecord->snap.nameServers();

		if (serversBefore == serversAfter)
		{
			continue;
		}

		std::wcout << L"Interface " << interfaceGuid << L" has been updated" << std::endl;
		std::wcout << L"before:" << std::endl;

		for (const auto &server : serversBefore)
		{
			std::wcout << L"    " << server << std::endl;
		}

		std::wcout << L"after:" << std::endl;

		for (const auto &server : serversAfter)
		{
			std::wcout << L"    " << server << std::endl;
		}
	}
}

void VerifySnapshots(const std::vector<InterfaceData> &v4, const std::vector<InterfaceData> &v6)
{
	std::vector<InterfaceData> updatedV4, updatedV6;

	CreateSnapshots(updatedV4, updatedV6);

	VerifyProtocolSnapshots(v4, updatedV4);
	VerifyProtocolSnapshots(v6, updatedV6);
}

int main()
{
	std::vector<InterfaceData> v4records, v6records;

	for (;;)
	{
		std::wcout << L"(T)ake interface settings snapshot" << std::endl
			<< L"(C)ompare current settings to snapshot" << std::endl
			<< L"(Q)uit" << std::endl;

		std::wcout << L"?: ";

		auto answer = _getwch();

		std::wcout << std::endl;

		//
		// Branch on selected command.
		//

		if ('t' == towlower(answer))
		{
			CreateSnapshots(v4records, v6records);

			std::wcout << L"Created new snapshot" << std::endl;

			continue;
		}

		if ('c' == towlower(answer))
		{
			VerifySnapshots(v4records, v6records);

			std::wcout << L"Comparison completed" << std::endl;

			continue;
		}

		if ('q' == towlower(answer))
		{
			break;
		}

		std::wcout << L"Unrecognized option" << std::endl;
	}

    return 0;
}
