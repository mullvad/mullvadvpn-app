#include "stdafx.h"
#include "deinit.h"
#include "winfw/winfw.h"
#include <libcommon/error.h>

namespace commands::winfw
{

Deinit::Deinit(MessageSink messageSink)
	: m_messageSink(messageSink)
{
}

std::wstring Deinit::name()
{
	return L"deinit";
}

std::wstring Deinit::description()
{
	return L"Deinitialize winfw; Destroy session and all associated objects";
}

void Deinit::handleRequest(const std::vector<std::wstring> &arguments)
{
	if (false == arguments.empty())
	{
		THROW_ERROR("Invalid argument(s). Cannot complete request.");
	}

	m_messageSink((WinFw_Deinitialize(WINFW_CLEANUP_POLICY_RESET_FIREWALL)
		? L"Deinitialization completed successfully."
		: L"Deinitialization failed. See above for details, if any."));
}

}
