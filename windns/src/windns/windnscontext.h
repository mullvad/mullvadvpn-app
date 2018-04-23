#pragma once

#include "windns.h"
#include <vector>
#include <string>

class WinDnsContext
{
public:

	WinDnsContext();

	bool set(const std::vector<std::wstring> &servers, WinDnsErrorSink errorSink, void *errorContext);
	bool reset();

//	static unsigned monitoringThread()

private:

	struct ThreadArguments
	{
		WinDnsContext *instance;
		WinDnsErrorSink errorSink;
		void *errorContext;
	};

};
