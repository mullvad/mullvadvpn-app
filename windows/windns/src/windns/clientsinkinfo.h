#pragma once

#include "windns.h"

struct LogSinkInfo
{
	WinDnsLogSink sink;
	void *context;
};

struct RecoverySinkInfo
{
	WinDnsRecoverySink sink;
	void *context;
};
