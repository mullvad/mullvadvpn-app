#include "stdafx.h"
#include "confineoperation.h"
#include "netsh.h"

bool ConfineOperation
(
	const char *literalOperation,
	std::shared_ptr<common::logging::ILogSink> logSink,
	std::function<void()> operation
)
{
	try
	{
		operation();
		return true;
	}
	catch (const std::exception &err)
	{
		const auto what = std::string(literalOperation).append(": ").append(err.what());

		logSink->error(what.c_str());

		return false;
	}
	catch (...)
	{
		const auto what = std::string(literalOperation).append(": Unspecified failure");

		logSink->error(what.c_str());

		return false;
	}
}
