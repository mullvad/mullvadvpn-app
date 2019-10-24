#include "stdafx.h"
#include "testadapterutil.h"
#include <iostream>

#include <CppUnitTest.h>
#include <libcommon/trace/trace.h>

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

		Assert::AreEqual(
			0ULL,
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
			1ULL,
			adapterCount,
			L"Expected new adapter"
		);
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
			L"Expected new adapter"
		);

		//
		// Register the same interface twice
		//
		testProvider->sendEvent(&iface, MibAddInstance);

		Assert::AreEqual(
			1ULL,
			adapterCount,
			L"Expected ignored duplicate interface event"
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
			0ULL,
			adapterCount,
			L"Expected removed adapter"
		);
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
			0ULL,
			adapterCount,
			L"Expected removed adapter"
		);
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
			1ULL,
			adapterCount,
			L"Expected new adapter"
		);
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
};
