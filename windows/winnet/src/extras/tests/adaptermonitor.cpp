#include "stdafx.h"
#include "testadapterutil.h"
#include <libshared/logging/stdoutlogger.h>
#include <libshared/logging/logsinkadapter.h>
#include <iostream>
#include <CppUnitTest.h>

using namespace Microsoft::VisualStudio::CppUnitTestFramework;


namespace
{

auto MakeStdoutLogger()
{
	return std::make_shared<shared::logging::LogSinkAdapter>(shared::logging::StdoutLogger, nullptr);
}
	
enum class LastEvent
{
	NoEvent,
	Add,
	Delete,
	Update
};

}

namespace Microsoft::VisualStudio::CppUnitTestFramework
{

template<>
std::wstring ToString<LastEvent>(const enum class LastEvent& t)
{
	switch (t)
	{
		case LastEvent::NoEvent:
			return L"LastEvent::NoEvent";
		case LastEvent::Add:
			return L"LastEvent::Add";
		case LastEvent::Delete:
			return L"LastEvent::Delete";
		case LastEvent::Update:
			return L"LastEvent::Update";
	}
	return L"LastEvent::<Unknown value>";
}

}

TEST_CLASS(NetworkAdapterMonitorTests)
{
public:
		
	TEST_METHOD(addAdapter)
	{
		auto logSink = MakeStdoutLogger();

		const auto filter = [](const MIB_IF_ROW2 &) -> bool
		{
			return true;
		};

		const auto testProvider = std::make_shared<TestDataProvider>();
		size_t adapterCount = 0;
		
		NetworkAdapterMonitor inst(
			logSink,
			[&adapterCount](const std::vector<MIB_IF_ROW2> &adapters, const MIB_IF_ROW2 *, UpdateType) -> void
			{
				adapterCount = adapters.size();
			},
			filter,
			testProvider
		);

		Assert::AreEqual(
			size_t(0),
			adapterCount,
			L"Expected 0 adapters initially"
		);

		//
		// Add new adapter
		//

		constexpr size_t currentLuid = 100;
		
		MIB_IF_ROW2 adapter = { 0 };
		adapter.InterfaceLuid.Value = currentLuid;
		adapter.AdminStatus = NET_IF_ADMIN_STATUS_UP;

		MIB_IPINTERFACE_ROW iface = { 0 };
		iface.InterfaceLuid.Value = currentLuid;
		iface.Family = AF_INET;

		testProvider->addIpInterface(adapter, iface);

		testProvider->sendEvent(&iface, MibAddInstance);

		Assert::AreEqual(
			size_t(1),
			adapterCount,
			L"Expected new adapter"
		);
	}

	TEST_METHOD(addAdapter_Duplicate)
	{
		auto logSink = MakeStdoutLogger();

		const auto filter = [](const MIB_IF_ROW2 &) -> bool
		{
			return true;
		};

		const auto testProvider = std::make_shared<TestDataProvider>();
		size_t adapterCount = 0;

		NetworkAdapterMonitor inst(
			logSink,
			[&adapterCount](const std::vector<MIB_IF_ROW2> &adapters, const MIB_IF_ROW2 *, UpdateType) -> void
			{
				adapterCount = adapters.size();
			},
			filter,
			testProvider
			);

		//
		// Create a fake adapter
		//

		MIB_IF_ROW2 adapter = { 0 };
		adapter.InterfaceLuid.Value = 1;
		adapter.AdminStatus = NET_IF_ADMIN_STATUS_UP;

		MIB_IPINTERFACE_ROW iface = { 0 };
		iface.InterfaceLuid.Value = 1;
		iface.Family = AF_INET;

		testProvider->addIpInterface(adapter, iface);

		testProvider->sendEvent(&iface, MibAddInstance);

		Assert::AreEqual(
			size_t(1),
			adapterCount,
			L"Expected new adapter"
		);

		//
		// Register the same interface twice
		//
		testProvider->sendEvent(&iface, MibAddInstance);

		Assert::AreEqual(
			size_t(1),
			adapterCount,
			L"Expected ignored duplicate interface event"
		);
	}

	TEST_METHOD(removeAdapter_AdminStatus)
	{
		auto logSink = MakeStdoutLogger();

		const auto filter = [](const MIB_IF_ROW2 &) -> bool
		{
			return true;
		};

		const auto testProvider = std::make_shared<TestDataProvider>();
		size_t adapterCount = 0;

		NetworkAdapterMonitor inst(
			logSink,
			[&adapterCount](const std::vector<MIB_IF_ROW2> &adapters, const MIB_IF_ROW2 *, UpdateType) -> void
			{
				adapterCount = adapters.size();
			},
			filter,
			testProvider
		);

		//
		// Create fake adapter
		//
		constexpr size_t luidValue = 1;
			
		MIB_IF_ROW2 adapter = { 0 };
		adapter.InterfaceLuid.Value = luidValue;
		adapter.AdminStatus = NET_IF_ADMIN_STATUS_UP;

		MIB_IPINTERFACE_ROW iface = { 0 };
		iface.InterfaceLuid.Value = luidValue;
		iface.Family = AF_INET;

		testProvider->addIpInterface(adapter, iface);

		testProvider->sendEvent(&iface, MibAddInstance);

		Assert::AreEqual(
			luidValue,
			adapterCount,
			L"Expected new adapter"
		);

		//
		// Delete adapter (AdminStatus)
		//
		adapter.AdminStatus = NET_IF_ADMIN_STATUS_DOWN;
		testProvider->addAdapter(adapter);

		testProvider->sendEvent(&iface, MibDeleteInstance);

		testProvider->removeAdapter(adapter);

		Assert::AreEqual(
			size_t(0),
			adapterCount,
			L"Expected removed adapter"
		);
	}

	TEST_METHOD(removeAdapter_NoInterfaces)
	{
		auto logSink = MakeStdoutLogger();

		const auto filter = [](const MIB_IF_ROW2 &) -> bool
		{
			return true;
		};

		const auto testProvider = std::make_shared<TestDataProvider>();
		size_t adapterCount = 0;

		NetworkAdapterMonitor inst(
			logSink,
			[&adapterCount](const std::vector<MIB_IF_ROW2> &adapters, const MIB_IF_ROW2 *, UpdateType) -> void
			{
				adapterCount = adapters.size();
			},
			filter,
			testProvider
		);

		//
		// Create fake adapter
		//

		constexpr size_t luidValue = 1;
		
		MIB_IF_ROW2 adapter = { 0 };
		adapter.InterfaceLuid.Value = luidValue;
		adapter.AdminStatus = NET_IF_ADMIN_STATUS_UP;

		MIB_IPINTERFACE_ROW iface = { 0 };
		iface.InterfaceLuid.Value = luidValue;
		iface.Family = AF_INET;

		testProvider->addIpInterface(adapter, iface);

		testProvider->sendEvent(&iface, MibAddInstance);

		Assert::AreEqual(
			luidValue,
			adapterCount,
			L"Expected new adapter (with IPv4 interface)"
		);

		//
		// Delete IP interfaces on the adapter (should report Delete)
		//

		testProvider->removeIpInterface(iface);
		testProvider->sendEvent(&iface, MibDeleteInstance);

		adapter = { 0 };
		adapter.InterfaceLuid.Value = luidValue;
		testProvider->removeAdapter(adapter);

		Assert::AreEqual(
			size_t(0),
			adapterCount,
			L"Expected removed adapter"
		);
	}

	TEST_METHOD(removeAdapter_Duplicate)
	{
		auto logSink = MakeStdoutLogger();

		const auto filter = [](const MIB_IF_ROW2 &) -> bool
		{
			return true;
		};

		const auto testProvider = std::make_shared<TestDataProvider>();
		size_t adapterCount = 0;

		NetworkAdapterMonitor inst(
			logSink,
			[&adapterCount](const std::vector<MIB_IF_ROW2> &adapters, const MIB_IF_ROW2 *, UpdateType) -> void
			{
				adapterCount = adapters.size();
			},
			filter,
			testProvider
		);

		//
		// Create fake adapter
		//
		MIB_IF_ROW2 adapter = { 0 };
		adapter.InterfaceLuid.Value = 1;
		adapter.AdminStatus = NET_IF_ADMIN_STATUS_UP;

		MIB_IPINTERFACE_ROW iface = { 0 };
		iface.InterfaceLuid.Value = 1;
		iface.Family = AF_INET;

		testProvider->addIpInterface(adapter, iface);

		testProvider->sendEvent(&iface, MibAddInstance);

		Assert::AreEqual(
			size_t(1),
			adapterCount,
			L"Expected new adapter"
		);

		//
		// Duplicate deletion events
		//

		adapter.AdminStatus = NET_IF_ADMIN_STATUS_DOWN;
		testProvider->addAdapter(adapter); // update status

		testProvider->removeIpInterface(iface);
		
		testProvider->sendEvent(&iface, MibDeleteInstance);
		testProvider->sendEvent(&iface, MibDeleteInstance);
		testProvider->sendEvent(&iface, MibDeleteInstance);
		testProvider->sendEvent(&iface, MibDeleteInstance);

		Assert::AreEqual(
			size_t(0),
			adapterCount,
			L"State inconsistent after duplicate Delete event"
		);
	}

	TEST_METHOD(addIPv6Interface)
	{
		auto logSink = MakeStdoutLogger();

		const auto filter = [](const MIB_IF_ROW2 &) -> bool
		{
			return true;
		};

		const auto testProvider = std::make_shared<TestDataProvider>();
		size_t adapterCount = 0;

		NetworkAdapterMonitor inst(
			logSink,
			[&adapterCount](const std::vector<MIB_IF_ROW2> &adapters, const MIB_IF_ROW2 *, UpdateType) -> void
			{
				adapterCount = adapters.size();
			},
			filter,
			testProvider
		);

		//
		// Add IPv6 interface
		//

		constexpr size_t currentLuid = 100;

		MIB_IF_ROW2 adapter = { 0 };
		adapter.InterfaceLuid.Value = currentLuid;
		adapter.AdminStatus = NET_IF_ADMIN_STATUS_UP;

		MIB_IPINTERFACE_ROW iface = { 0 };
		iface.InterfaceLuid.Value = currentLuid;
		iface.Family = AF_INET6;

		testProvider->addIpInterface(adapter, iface);

		testProvider->sendEvent(&iface, MibAddInstance);

		Assert::AreEqual(
			size_t(1),
			adapterCount,
			L"Expected new adapter"
		);
	}

	TEST_METHOD(addIPv4And6Interface)
	{
		auto logSink = MakeStdoutLogger();

		const auto filter = [](const MIB_IF_ROW2 &) -> bool
		{
			return true;
		};

		const auto testProvider = std::make_shared<TestDataProvider>();
		size_t adapterCount = 0;

		NetworkAdapterMonitor inst(
			logSink,
			[&adapterCount](const std::vector<MIB_IF_ROW2> &adapters, const MIB_IF_ROW2 *, UpdateType) -> void
			{
				adapterCount = adapters.size();
			},
			filter,
			testProvider
		);

		constexpr size_t currentLuid = 1;
		constexpr size_t expectedCount = 1;

		MIB_IF_ROW2 adapter = { 0 };
		adapter.InterfaceLuid.Value = currentLuid;
		adapter.AdminStatus = NET_IF_ADMIN_STATUS_UP;

		//
		// Add IPv4 interface
		//
		
		MIB_IPINTERFACE_ROW iface4 = { 0 };
		iface4.InterfaceLuid.Value = currentLuid;
		iface4.Family = AF_INET;
		testProvider->addIpInterface(adapter, iface4);
		testProvider->sendEvent(&iface4, MibAddInstance);

		//
		// Add IPv6 interface
		//

		MIB_IPINTERFACE_ROW iface6 = { 0 };
		iface6.InterfaceLuid.Value = currentLuid;
		iface6.Family = AF_INET6;
		testProvider->addIpInterface(adapter, iface6);
		testProvider->sendEvent(&iface6, MibAddInstance);

		Assert::AreEqual(
			expectedCount,
			adapterCount,
			L"Expected single adapter with two IP interfaces"
		);
	}

	TEST_METHOD(addIPv4And6Interface_RemoveIPv4)
	{
		auto logSink = MakeStdoutLogger();

		const auto filter = [](const MIB_IF_ROW2 &) -> bool
		{
			return true;
		};

		const auto testProvider = std::make_shared<TestDataProvider>();
		size_t adapterCount = 0;

		NetworkAdapterMonitor inst(
			logSink,
			[&adapterCount](const std::vector<MIB_IF_ROW2> &adapters, const MIB_IF_ROW2 *, UpdateType) -> void
			{
				adapterCount = adapters.size();
			},
			filter,
			testProvider
		);

		constexpr size_t currentLuid = 1;

		MIB_IF_ROW2 adapter = { 0 };
		adapter.InterfaceLuid.Value = currentLuid;
		adapter.AdminStatus = NET_IF_ADMIN_STATUS_UP;

		//
		// Add IPv4 interface
		//

		MIB_IPINTERFACE_ROW iface4 = { 0 };
		iface4.InterfaceLuid.Value = currentLuid;
		iface4.Family = AF_INET;
		testProvider->addIpInterface(adapter, iface4);
		testProvider->sendEvent(&iface4, MibAddInstance);

		//
		// Add IPv6 interface
		//

		MIB_IPINTERFACE_ROW iface6 = { 0 };
		iface6.InterfaceLuid.Value = currentLuid;
		iface6.Family = AF_INET6;
		testProvider->addIpInterface(adapter, iface6);
		testProvider->sendEvent(&iface6, MibAddInstance);

		//
		// Remove IPv4 interface
		//
		testProvider->removeIpInterface(iface4);
		testProvider->sendEvent(&iface4, MibDeleteInstance);

		constexpr size_t expectedCount = 1;

		Assert::AreEqual(
			expectedCount,
			adapterCount,
			L"Expected single adapter (with IPv6 interface)"
		);
	}

	TEST_METHOD(addIPv4And6Interface_RemoveIPv6)
	{
		auto logSink = MakeStdoutLogger();

		const auto filter = [](const MIB_IF_ROW2 &) -> bool
		{
			return true;
		};

		const auto testProvider = std::make_shared<TestDataProvider>();
		size_t adapterCount = 0;

		NetworkAdapterMonitor inst(
			logSink,
			[&adapterCount](const std::vector<MIB_IF_ROW2> &adapters, const MIB_IF_ROW2 *, UpdateType) -> void
			{
				adapterCount = adapters.size();
			},
			filter,
			testProvider
		);

		constexpr size_t currentLuid = 1;

		MIB_IF_ROW2 adapter = { 0 };
		adapter.InterfaceLuid.Value = currentLuid;
		adapter.AdminStatus = NET_IF_ADMIN_STATUS_UP;

		//
		// Add IPv4 interface
		//

		MIB_IPINTERFACE_ROW iface4 = { 0 };
		iface4.InterfaceLuid.Value = currentLuid;
		iface4.Family = AF_INET;
		testProvider->addIpInterface(adapter, iface4);
		testProvider->sendEvent(&iface4, MibAddInstance);

		//
		// Add IPv6 interface
		//

		MIB_IPINTERFACE_ROW iface6 = { 0 };
		iface6.InterfaceLuid.Value = currentLuid;
		iface6.Family = AF_INET6;
		testProvider->addIpInterface(adapter, iface6);
		testProvider->sendEvent(&iface6, MibAddInstance);

		//
		// Remove IPv6 interface
		//
		testProvider->removeIpInterface(iface6);
		testProvider->sendEvent(&iface6, MibDeleteInstance);

		constexpr size_t expectedCount = 1;

		Assert::AreEqual(
			expectedCount,
			adapterCount,
			L"Expected single adapter (with IPv4 interface)"
		);
	}

	TEST_METHOD(addIPv4And6Interface_RemoveBoth)
	{
		auto logSink = MakeStdoutLogger();

		const auto filter = [](const MIB_IF_ROW2 &) -> bool
		{
			return true;
		};

		const auto testProvider = std::make_shared<TestDataProvider>();
		size_t adapterCount = 0;

		NetworkAdapterMonitor inst(
			logSink,
			[&adapterCount](const std::vector<MIB_IF_ROW2> &adapters, const MIB_IF_ROW2 *, UpdateType) -> void
			{
				adapterCount = adapters.size();
			},
			filter,
			testProvider
		);

		constexpr size_t currentLuid = 1;

		MIB_IF_ROW2 adapter = { 0 };
		adapter.InterfaceLuid.Value = currentLuid;
		adapter.AdminStatus = NET_IF_ADMIN_STATUS_UP;

		//
		// Add IPv4 interface
		//

		MIB_IPINTERFACE_ROW iface4 = { 0 };
		iface4.InterfaceLuid.Value = currentLuid;
		iface4.Family = AF_INET;
		testProvider->addIpInterface(adapter, iface4);
		testProvider->sendEvent(&iface4, MibAddInstance);

		//
		// Add IPv6 interface
		//

		MIB_IPINTERFACE_ROW iface6 = { 0 };
		iface6.InterfaceLuid.Value = currentLuid;
		iface6.Family = AF_INET6;
		testProvider->addIpInterface(adapter, iface6);
		testProvider->sendEvent(&iface6, MibAddInstance);

		//
		// Remove IP interfaces
		//
		testProvider->removeIpInterface(iface4);
		testProvider->sendEvent(&iface4, MibDeleteInstance);
		testProvider->removeIpInterface(iface6);
		testProvider->sendEvent(&iface6, MibDeleteInstance);

		constexpr size_t expectedCount = 0;

		Assert::AreEqual(
			expectedCount,
			adapterCount,
			L"Expected no adapter (0 IP interfaces)"
		);
	}

	TEST_METHOD(filter)
	{
		auto logSink = MakeStdoutLogger();

		const auto testProvider = std::make_shared<TestDataProvider>();

		//
		// Exclude adapters not connected to the internet,
		// loopback devices, and software adapters
		//

		const auto filter = [](const MIB_IF_ROW2 &row) -> bool
		{
			switch (row.InterfaceLuid.Info.IfType)
			{
				case IF_TYPE_SOFTWARE_LOOPBACK:
				{
					return false;
				}
			}

			if (FALSE == row.InterfaceAndOperStatusFlags.HardwareInterface)
			{
				return false;
			}
			return IfOperStatusUp == row.OperStatus
				&& MediaConnectStateConnected == row.MediaConnectState;
		};

		size_t adapterCount = 0;
		LastEvent lastEvent = LastEvent::NoEvent;

		NetworkAdapterMonitor inst(
			logSink,
			[&adapterCount, &lastEvent](const std::vector<MIB_IF_ROW2> &adapters, const MIB_IF_ROW2 *, UpdateType updateType) -> void
			{
				switch (updateType)
				{
					case UpdateType::Add:
						lastEvent = LastEvent::Add;
						break;
					case UpdateType::Delete:
						lastEvent = LastEvent::Delete;
						break;
					case UpdateType::Update:
						lastEvent = LastEvent::Update;
						break;
					default:
						Assert::Fail(L"Unhandled update type");
				}
			
				adapterCount = adapters.size();
			},
			filter,
			testProvider
		);

		//
		// Our filter should ignore loopback devices
		//

		constexpr size_t loopbackLuid = 1;
		
		MIB_IF_ROW2 adapter = { 0 };
		adapter.AdminStatus = NET_IF_ADMIN_STATUS_UP;
		adapter.InterfaceLuid.Value = loopbackLuid;
		adapter.InterfaceLuid.Info.IfType = IF_TYPE_SOFTWARE_LOOPBACK;
		adapter.MediaConnectState = MediaConnectStateConnected;
		adapter.InterfaceAndOperStatusFlags.HardwareInterface = TRUE;
		adapter.OperStatus = IfOperStatusUp;

		MIB_IPINTERFACE_ROW iface4 = { 0 };
		iface4.InterfaceLuid.Value = loopbackLuid;
		iface4.Family = AF_INET;

		lastEvent = LastEvent::NoEvent;
		
		testProvider->addIpInterface(adapter, iface4);
		testProvider->sendEvent(&iface4, MibAddInstance);

		Assert::AreEqual(LastEvent::NoEvent, lastEvent, L"Unexpectedly received event for loopback adapter");

		Assert::AreEqual(
			size_t(0),
			adapterCount,
			L"Loopback adapter was not filtered correctly"
		);

		testProvider->removeIpInterface(iface4);
		testProvider->sendEvent(&iface4, MibDeleteInstance);
		testProvider->removeAdapter(adapter);

		Assert::AreEqual(LastEvent::NoEvent, lastEvent, L"Unexpectedly received event for loopback adapter");

		//
		// Our filter should ignore devices not connected to the internet
		//

		constexpr size_t disconnectedLuid = 2;

		adapter = { 0 };
		adapter.AdminStatus = NET_IF_ADMIN_STATUS_UP;
		adapter.InterfaceLuid.Value = disconnectedLuid;
		adapter.MediaConnectState = MediaConnectStateDisconnected;
		adapter.InterfaceAndOperStatusFlags.HardwareInterface = TRUE;
		adapter.OperStatus = IfOperStatusUp;

		iface4 = { 0 };
		iface4.InterfaceLuid.Value = disconnectedLuid;
		iface4.Family = AF_INET;
		testProvider->addIpInterface(adapter, iface4);
		testProvider->sendEvent(&iface4, MibAddInstance);

		Assert::AreEqual(LastEvent::NoEvent, lastEvent, L"Unexpectedly received event for disconnected adapter");

		testProvider->removeIpInterface(iface4);
		testProvider->sendEvent(&iface4, MibDeleteInstance);
		testProvider->removeAdapter(adapter);

		Assert::AreEqual(LastEvent::NoEvent, lastEvent, L"Unexpectedly received event for disconnected adapter");

		//
		// Report events for hardware devices
		//

		constexpr size_t onlineHardwareLuid = 3;

		adapter = { 0 };
		adapter.AdminStatus = NET_IF_ADMIN_STATUS_UP;
		adapter.InterfaceLuid.Value = onlineHardwareLuid;
		adapter.MediaConnectState = MediaConnectStateConnected;
		adapter.InterfaceAndOperStatusFlags.HardwareInterface = TRUE;
		adapter.OperStatus = IfOperStatusUp;

		iface4 = { 0 };
		iface4.InterfaceLuid.Value = onlineHardwareLuid;
		iface4.Family = AF_INET;
		testProvider->addIpInterface(adapter, iface4);
		testProvider->sendEvent(&iface4, MibAddInstance);

		Assert::AreEqual(LastEvent::Add, lastEvent, L"Expected event for connected adapter was not received");

		lastEvent = LastEvent::NoEvent;
		
		testProvider->removeIpInterface(iface4);
		testProvider->sendEvent(&iface4, MibDeleteInstance);
		testProvider->removeAdapter(adapter);

		Assert::AreEqual(LastEvent::Delete, lastEvent, L"Expected event for connected adapter was not received");
	}
};
