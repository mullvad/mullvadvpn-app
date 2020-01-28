#include "stdafx.h"
#include "init.h"
#include <libcommon/string.h>
#include <libcommon/error.h>

namespace commands::winfw
{

Init::Init(MessageSink messageSink)
	: m_messageSink(messageSink)
{
}

std::wstring Init::name()
{
	return L"init";
}

std::wstring Init::description()
{
	return L"Initialize winfw; Create session and fundamental objects";
}

void Init::handleRequest(const std::vector<std::wstring> &arguments)
{
	uint32_t timeout = 0;

	if (false == arguments.empty())
	{
		auto keyvalue = common::string::SplitKeyValuePairs(arguments);

		if (keyvalue.empty() || 0 != keyvalue.begin()->first.compare(L"timeout"))
		{
			THROW_ERROR("Invalid argument. Cannot complete request.");
		}

		timeout = wcstoul(keyvalue.begin()->second.c_str(), nullptr, 10);
	}

	auto success = WinFw_Initialize(timeout, &Init::ErrorForwarder, this);

	m_messageSink((success
		? L"Initialization completed successfully."
		: L"Initialization failed. See above for details, if any."));
}

//static
void WINFW_API Init::ErrorForwarder(MULLVAD_LOG_LEVEL, const char *errorMessage, void *context)
{
	auto thiz = reinterpret_cast<Init *>(context);

	thiz->m_messageSink(common::string::ToWide(errorMessage));
}

}
