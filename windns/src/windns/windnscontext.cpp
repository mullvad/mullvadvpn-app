#include "stdafx.h"
#include "windnscontext.h"
#include "wmi/connection.h"
#include <process.h>

WinDnsContext::WinDnsContext()
{
	// Create WMI connection and keep it around?
}

bool WinDnsContext::set(const std::vector<std::wstring> &servers, WinDnsErrorSink errorSink, void *errorContext)
{
	ThreadArguments args;

	args.errorSink = errorSink;
	args.errorContext = errorContext;

//	_beginthreadex(nullptr, 0, )



	return false;
}

bool WinDnsContext::reset()
{
	return false;
}
