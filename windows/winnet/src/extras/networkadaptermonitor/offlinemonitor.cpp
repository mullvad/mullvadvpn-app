#include "stdafx.h"
#include <WinSock2.h>
#include "CppUnitTest.h"
#include <iostream>
#include "libcommon/trace/trace.h"
#include "testadapterutil.h"

using namespace Microsoft::VisualStudio::CppUnitTestFramework;

#include "libcommon/logging/logsink.h"
#include "../../winnet/offlinemonitor.h"

using FilterType = NetworkAdapterMonitor::FilterType;
using UpdateSinkType = NetworkAdapterMonitor::UpdateSinkType;
using UpdateType = NetworkAdapterMonitor::UpdateType;

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

TEST_CLASS(OfflineMonitorTests)
{
public:

	TEST_METHOD(construct)
	{
		auto logSink = std::shared_ptr<common::logging::ILogSink>();
		logSink.reset(new common::logging::LogSink(logFunc));

		bool isConnected = false;

		const auto statusNotifier = [&isConnected](bool connected) -> void
		{
			isConnected = connected;
		};

		auto testNotifier = std::make_shared<TestWinNotifier>();
		
		OfflineMonitor inst(
			logSink,
			statusNotifier,
			testNotifier,
			[](std::map<ULONG64, MIB_IF_ROW2> &adaptersOut)
			{
			}
		);

		Assert::IsFalse(isConnected);
	}

	/*TEST_METHOD(connect)
	{
		// TODO: connect a fake adapter, verify that it works
	}*/

	/*TEST_METHOD(disconnect)
	{
		// TODO: add a connected interface, remove it. verify that the status is offline
	}*/
};
