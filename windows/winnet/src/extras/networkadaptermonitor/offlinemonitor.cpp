#include "stdafx.h"
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

