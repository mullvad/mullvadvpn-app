#include "stdafx.h"
#include "deinit.h"
#include "wfpctl/wfpctl.h"

namespace commands::wfpctl
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
	return L"Deinitialize wfpctl; Destroy session and all associated objects";
}

void Deinit::handleRequest(const std::vector<std::wstring> &arguments)
{
	if (false == arguments.empty())
	{
		throw std::runtime_error("Invalid argument(s). Cannot complete request.");
	}

	m_messageSink((Wfpctl_Deinitialize()
		? L"Deinitialization completed successfully."
		: L"Deinitialization failed. See above for details, if any."));
}

}
