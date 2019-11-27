#pragma once

#include "logsink.h"
#include "logsinkadapter.h"
#include <libcommon/error.h>
#include <stdexcept>

namespace shared
{

void UnwindAndLog(MullvadLogSink logSink, void *logSinkContext, const std::exception &err)
{
	if (nullptr == logSink)
	{
		return;
	}

	auto logger = std::make_shared<shared::LogSinkAdapter>(logSink, logSinkContext);

	common::error::UnwindException(err, logger);
}

}
