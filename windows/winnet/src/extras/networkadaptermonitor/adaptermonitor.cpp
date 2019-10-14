#include "stdafx.h"
#include "testadapterutil.h"
#include <iostream>

#include "CppUnitTest.h"
#include "libcommon/trace/trace.h"

using namespace Microsoft::VisualStudio::CppUnitTestFramework;


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

TEST_CLASS(NetworkAdapterMonitorTests)
{
public:
		
	TEST_METHOD(addAdapter)
	{
		auto logSink = std::shared_ptr<common::logging::ILogSink>();
		logSink.reset(new common::logging::LogSink(logFunc));

		const auto filter = [](const MIB_IF_ROW2 &row) -> bool
		{
			return true;
		};

		const auto testNotifier = std::make_shared<TestWinNotifier>();
		NetworkAdapterMonitorTester inst(
			logSink,
			filter,
			testNotifier
		);

		//
		// Create a fake adapter
		//
		MIB_IPINTERFACE_ROW row = { 0 };
		row.InterfaceLuid.Value = 14195234ULL;
		row.Family = AF_INET;

		testNotifier->sendEvent(&row, MibAddInstance);

		Assert::AreEqual(
			1ULL,
			inst.getAdapters().size(),
			L"Adapter not added"
		);

		row.InterfaceLuid.Value = 14195235ULL;
		testNotifier->sendEvent(&row, MibAddInstance);

		Assert::AreEqual(
			2ULL,
			inst.getAdapters().size(),
			L"Adapter not added"
		);
	}

	TEST_METHOD(addDuplicates)
	{
		auto logSink = std::shared_ptr<common::logging::ILogSink>();
		logSink.reset(new common::logging::LogSink(logFunc));

		const auto filter = [](const MIB_IF_ROW2 &row) -> bool
		{
			return true;
		};

		const auto testNotifier = std::make_shared<TestWinNotifier>();
		NetworkAdapterMonitorTester inst(
			logSink,
			filter,
			testNotifier
		);

		//
		// Create a fake adapter
		//
		MIB_IPINTERFACE_ROW row = { 0 };
		row.InterfaceLuid.Value = 14195234ULL;
		row.Family = AF_INET;

		testNotifier->sendEvent(&row, MibAddInstance);
		testNotifier->sendEvent(&row, MibAddInstance);
		testNotifier->sendEvent(&row, MibAddInstance);

		Assert::AreEqual(
			1ULL,
			inst.getAdapters().size(),
			L"Duplicates not ignored"
		);
	}

	TEST_METHOD(removeAdapter)
	{
		auto logSink = std::shared_ptr<common::logging::ILogSink>();
		logSink.reset(new common::logging::LogSink(logFunc));

		const auto filter = [](const MIB_IF_ROW2 &row) -> bool
		{
			return true;
		};

		const auto testNotifier = std::make_shared<TestWinNotifier>();
		NetworkAdapterMonitorTester inst(
			logSink,
			filter,
			testNotifier
		);

		//
		// Create a fake adapter
		//
		MIB_IPINTERFACE_ROW row = { 0 };
		row.InterfaceLuid.Value = 14195234ULL;
		row.Family = AF_INET;

		testNotifier->sendEvent(&row, MibAddInstance);
		testNotifier->sendEvent(&row, MibAddInstance);
		testNotifier->sendEvent(&row, MibDeleteInstance);
		testNotifier->sendEvent(&row, MibDeleteInstance);
		testNotifier->sendEvent(&row, MibDeleteInstance);

		Assert::AreEqual(
			0ULL,
			inst.getAdapters().size(),
			L"Delete event inconsistent"
		);
	}
};
