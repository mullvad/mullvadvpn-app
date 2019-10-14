#include "stdafx.h"

#include "testadapterutil.h"

#include <WinSock2.h>
#include "CppUnitTest.h"
#include <iostream>
#include "libcommon/trace/trace.h"

using namespace Microsoft::VisualStudio::CppUnitTestFramework;

#include "libcommon/logging/logsink.h"
#include "../../winnet/networkadaptermonitor.h"

using FilterType = NetworkAdapterMonitor::FilterType;
using UpdateSinkType = NetworkAdapterMonitor::UpdateSinkType;
using UpdateType = NetworkAdapterMonitor::UpdateType;


class NetworkAdapterMonitorTester;

//
// Notifier tester
//

void TestWinNotifier::attach(std::shared_ptr<common::logging::ILogSink> logSink, AdapterUpdate callback)
{
	m_logSink = logSink;
	m_callback = callback;
}

void TestWinNotifier::detach()
{
}

void TestWinNotifier::sendEvent(const MIB_IPINTERFACE_ROW* hint, MIB_NOTIFICATION_TYPE updateType) const
{
	m_callback(hint, updateType);
}

//
// NetworkAdapterMonitorTester
//

void NetworkAdapterMonitorTester::getIfEntry(MIB_IF_ROW2 &rowOut, NET_LUID luid)
{
	rowOut = { 0 };
	rowOut.InterfaceLuid = luid;
}
