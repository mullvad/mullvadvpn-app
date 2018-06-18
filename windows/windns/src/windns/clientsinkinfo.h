#pragma once

#include "windns.h"

struct ErrorSinkInfo
{
	WinDnsErrorSink sink;
	void *context;
};

struct ConfigSinkInfo
{
	WinDnsConfigSink sink;
	void *context;
};

struct ClientSinkInfo
{
	ErrorSinkInfo errorSinkInfo;
	ConfigSinkInfo configSinkInfo;
};
