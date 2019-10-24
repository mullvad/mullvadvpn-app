#include "stdafx.h"
#include "testadapterutil.h"
#include <iostream>

#include "CppUnitTest.h"
#include "libcommon/trace/trace.h"

using namespace Microsoft::VisualStudio::CppUnitTestFramework;


namespace
{
	
void logFunc(common::logging::Severity severity, const char *msg)
{
	using common::logging::Severity;
	
	switch (severity)
	{
	case Severity::Error:
		std::cout << "Error: ";
		break;
	case Severity::Warning:
		std::cout << "Warning: ";
		break;
	case Severity::Info:
		std::cout << "Info: ";
		break;
	case Severity::Trace:
		std::cout << "Trace: ";
		break;
	}

	std::cout << msg << std::endl;
}

}

TEST_CLASS(NetworkAdapterMonitorTests)
{
public:
		
	TEST_METHOD(addAdapter)
	{
		auto logSink = std::make_shared<common::logging::LogSink>(logFunc);

		const auto filter = [](const MIB_IF_ROW2 &row) -> bool
		{
			return true;
		};

		const auto testProvider = std::make_shared<TestDataProvider>();
		size_t adapterCount = 0;
		
		NetworkAdapterMonitor inst(
			logSink,
			[&adapterCount](const std::vector<MIB_IF_ROW2> &adapters, const MIB_IF_ROW2 *adapter, UpdateType updateType) -> void
			{
				adapterCount = adapters.size();
			},
			filter,
			testProvider
		);

		//
		// Add adapters
		//

		size_t expectedCount = adapterCount + 1;

		for (; expectedCount < 100; expectedCount++)
		{
			size_t currentLuid = expectedCount + 100;
			
			MIB_IF_ROW2 adapter = { 0 };
			adapter.InterfaceLuid.Value = currentLuid;
			adapter.AdminStatus = NET_IF_ADMIN_STATUS_UP;

			MIB_IPINTERFACE_ROW iface = { 0 };
			iface.InterfaceLuid.Value = currentLuid;
			iface.Family = AF_INET;

			testProvider->addIpInterface(adapter, iface);

			testProvider->sendEvent(&iface, MibAddInstance);

			Assert::AreEqual(
				expectedCount,
				adapterCount,
				L"Adapter not added"
			);
		}
	}

	TEST_METHOD(addAdapter_Duplicate)
	{
		auto logSink = std::make_shared<common::logging::LogSink>(logFunc);

		const auto filter = [](const MIB_IF_ROW2 &row) -> bool
		{
			return true;
		};

		const auto testProvider = std::make_shared<TestDataProvider>();
		size_t adapterCount = 0;

		NetworkAdapterMonitor inst(
			logSink,
			[&adapterCount](const std::vector<MIB_IF_ROW2> &adapters, const MIB_IF_ROW2 *adapter, UpdateType updateType) -> void
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
			1ULL,
			adapterCount,
			L"Adapter not added"
		);

		//
		// Register the same adapter twice
		//
		testProvider->sendEvent(&iface, MibAddInstance);

		Assert::AreEqual(
			1ULL,
			adapterCount,
			L"Duplicate was not ignored"
		);
	}

	TEST_METHOD(removeAdapter_AdminStatus)
	{
		auto logSink = std::make_shared<common::logging::LogSink>(logFunc);

		const auto filter = [](const MIB_IF_ROW2 &row) -> bool
		{
			return true;
		};

		const auto testProvider = std::make_shared<TestDataProvider>();
		size_t adapterCount = 0;

		NetworkAdapterMonitor inst(
			logSink,
			[&adapterCount](const std::vector<MIB_IF_ROW2> &adapters, const MIB_IF_ROW2 *adapter, UpdateType updateType) -> void
			{
				adapterCount = adapters.size();
			},
			filter,
			testProvider
		);

		constexpr size_t firstLuidValue = 1;
		constexpr size_t lastLuidValue = 1000;

		//
		// Create fake adapters
		//
		for (size_t luidValue = firstLuidValue; luidValue <= lastLuidValue; luidValue++)
		{
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
				L"Adapter not added"
			);
		}

		//
		// Delete adapters (AdminStatus)
		//
		size_t initialCount = adapterCount;
		size_t expectedCount = initialCount;

		for (size_t luidValue = firstLuidValue; luidValue <= lastLuidValue; luidValue++)
		{
			MIB_IF_ROW2 adapter = { 0 };
			adapter.InterfaceLuid.Value = luidValue;
			adapter.AdminStatus = NET_IF_ADMIN_STATUS_DOWN;

			testProvider->addAdapter(adapter);

			MIB_IPINTERFACE_ROW iface = { 0 };
			iface.InterfaceLuid.Value = luidValue;
			iface.Family = AF_INET;

			testProvider->sendEvent(&iface, MibDeleteInstance);

			testProvider->removeAdapter(adapter);

			expectedCount--;

			Assert::AreEqual(
				expectedCount,
				adapterCount,
				L"Adapter was not removed"
			);
		}
	}

	TEST_METHOD(removeAdapter_NoInterfaces)
	{
		auto logSink = std::make_shared<common::logging::LogSink>(logFunc);

		const auto filter = [](const MIB_IF_ROW2 &row) -> bool
		{
			return true;
		};

		const auto testProvider = std::make_shared<TestDataProvider>();
		size_t adapterCount = 0;

		NetworkAdapterMonitor inst(
			logSink,
			[&adapterCount](const std::vector<MIB_IF_ROW2> &adapters, const MIB_IF_ROW2 *adapter, UpdateType updateType) -> void
			{
				adapterCount = adapters.size();
			},
			filter,
			testProvider
		);

		constexpr size_t firstLuidValue = 1;
		constexpr size_t lastLuidValue = 1000;

		//
		// Create fake adapters
		//
		for (size_t luidValue = firstLuidValue; luidValue <= lastLuidValue; luidValue++)
		{
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
				L"Adapter not added"
			);
		}

		//
		// Delete IP interfaces on all adapters (should report Delete)
		//
		size_t initialCount = adapterCount;
		size_t expectedCount = initialCount;

		for (size_t luidValue = firstLuidValue; luidValue <= lastLuidValue; luidValue++)
		{
			MIB_IPINTERFACE_ROW iface = { 0 };
			iface.InterfaceLuid.Value = luidValue;
			iface.Family = AF_INET;

			testProvider->removeIpInterface(iface);

			testProvider->sendEvent(&iface, MibDeleteInstance);

			MIB_IF_ROW2 adapter = { 0 };
			adapter.InterfaceLuid.Value = luidValue;
			testProvider->removeAdapter(adapter);

			expectedCount--;

			Assert::AreEqual(
				expectedCount,
				adapterCount,
				L"Adapter was not removed"
			);
		}
	}

	TEST_METHOD(removeAdapter_Duplicate)
	{
		auto logSink = std::make_shared<common::logging::LogSink>(logFunc);

		const auto filter = [](const MIB_IF_ROW2 &row) -> bool
		{
			return true;
		};

		const auto testProvider = std::make_shared<TestDataProvider>();
		size_t adapterCount = 0;

		NetworkAdapterMonitor inst(
			logSink,
			[&adapterCount](const std::vector<MIB_IF_ROW2> &adapters, const MIB_IF_ROW2 *adapter, UpdateType updateType) -> void
			{
				adapterCount = adapters.size();
			},
			filter,
			testProvider
		);

		constexpr size_t firstLuidValue = 1;
		constexpr size_t lastLuidValue = 1000;

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
			1ULL,
			adapterCount,
			L"Adapter not added"
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
			0ULL,
			adapterCount,
			L"State inconsistent after duplicate Delete event"
		);
	}

	TEST_METHOD(addIPv6Interface)
	{
		auto logSink = std::make_shared<common::logging::LogSink>(logFunc);

		const auto filter = [](const MIB_IF_ROW2 &row) -> bool
		{
			return true;
		};

		const auto testProvider = std::make_shared<TestDataProvider>();
		size_t adapterCount = 0;

		NetworkAdapterMonitor inst(
			logSink,
			[&adapterCount](const std::vector<MIB_IF_ROW2> &adapters, const MIB_IF_ROW2 *adapter, UpdateType updateType) -> void
			{
				adapterCount = adapters.size();
			},
			filter,
			testProvider
		);

		//
		// Add multiple adapters
		//

		size_t expectedCount = adapterCount + 1;

		for (; expectedCount < 100; expectedCount++)
		{
			size_t currentLuid = expectedCount + 100;

			MIB_IF_ROW2 adapter = { 0 };
			adapter.InterfaceLuid.Value = currentLuid;
			adapter.AdminStatus = NET_IF_ADMIN_STATUS_UP;

			MIB_IPINTERFACE_ROW iface = { 0 };
			iface.InterfaceLuid.Value = currentLuid;
			iface.Family = AF_INET6;

			testProvider->addIpInterface(adapter, iface);

			testProvider->sendEvent(&iface, MibAddInstance);

			Assert::AreEqual(
				expectedCount,
				adapterCount,
				L"Adapter not added"
			);
		}
	}

	TEST_METHOD(addIPv4And6Interface)
	{
		auto logSink = std::make_shared<common::logging::LogSink>(logFunc);

		const auto filter = [](const MIB_IF_ROW2 &row) -> bool
		{
			return true;
		};

		const auto testProvider = std::make_shared<TestDataProvider>();
		size_t adapterCount = 0;

		NetworkAdapterMonitor inst(
			logSink,
			[&adapterCount](const std::vector<MIB_IF_ROW2> &adapters, const MIB_IF_ROW2 *adapter, UpdateType updateType) -> void
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
		auto logSink = std::make_shared<common::logging::LogSink>(logFunc);

		const auto filter = [](const MIB_IF_ROW2 &row) -> bool
		{
			return true;
		};

		const auto testProvider = std::make_shared<TestDataProvider>();
		size_t adapterCount = 0;

		NetworkAdapterMonitor inst(
			logSink,
			[&adapterCount](const std::vector<MIB_IF_ROW2> &adapters, const MIB_IF_ROW2 *adapter, UpdateType updateType) -> void
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
		auto logSink = std::make_shared<common::logging::LogSink>(logFunc);

		const auto filter = [](const MIB_IF_ROW2 &row) -> bool
		{
			return true;
		};

		const auto testProvider = std::make_shared<TestDataProvider>();
		size_t adapterCount = 0;

		NetworkAdapterMonitor inst(
			logSink,
			[&adapterCount](const std::vector<MIB_IF_ROW2> &adapters, const MIB_IF_ROW2 *adapter, UpdateType updateType) -> void
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
		auto logSink = std::make_shared<common::logging::LogSink>(logFunc);

		const auto filter = [](const MIB_IF_ROW2 &row) -> bool
		{
			return true;
		};

		const auto testProvider = std::make_shared<TestDataProvider>();
		size_t adapterCount = 0;

		NetworkAdapterMonitor inst(
			logSink,
			[&adapterCount](const std::vector<MIB_IF_ROW2> &adapters, const MIB_IF_ROW2 *adapter, UpdateType updateType) -> void
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
		auto logSink = std::make_shared<common::logging::LogSink>(logFunc);

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
		bool receivedEvent = false;

		NetworkAdapterMonitor inst(
			logSink,
			[&adapterCount, &receivedEvent](const std::vector<MIB_IF_ROW2> &adapters, const MIB_IF_ROW2 *adapter, UpdateType updateType) -> void
			{
				adapterCount = adapters.size();
				receivedEvent = true;
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
		testProvider->addIpInterface(adapter, iface4);
		testProvider->sendEvent(&iface4, MibAddInstance);

		Assert::IsFalse(receivedEvent, L"Unexpectedly received event for loopback adapter");

		Assert::AreEqual(
			0ULL,
			adapterCount,
			L"Loopback adapter was not filtered correctly"
		);

		testProvider->removeIpInterface(iface4);
		testProvider->sendEvent(&iface4, MibDeleteInstance);
		testProvider->removeAdapter(adapter);

		Assert::IsFalse(receivedEvent, L"Unexpectedly received event for loopback adapter");

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

		Assert::IsFalse(receivedEvent, L"Unexpectedly received event for disconnected adapter");

		testProvider->removeIpInterface(iface4);
		testProvider->sendEvent(&iface4, MibDeleteInstance);
		testProvider->removeAdapter(adapter);

		Assert::IsFalse(receivedEvent, L"Unexpectedly received event for disconnected adapter");

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

		Assert::IsTrue(receivedEvent, L"Expected Add event for connected adapter was not received");

		receivedEvent = false;
		
		testProvider->removeIpInterface(iface4);
		testProvider->sendEvent(&iface4, MibDeleteInstance);
		testProvider->removeAdapter(adapter);

		Assert::IsTrue(receivedEvent, L"Expected Delete event for connected adapter was not received");
	}
};
