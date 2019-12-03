#include "stdafx.h"
#include "unwind.h"
#include "logsinkadapter.h"
#include <libcommon/error.h>

namespace shared::logging
{

void UnwindAndLog(MullvadLogSink logSink, void *logSinkContext, const std::exception &err)
{
	if (nullptr == logSink)
	{
		return;
	}

	auto logger = std::make_shared<LogSinkAdapter>(logSink, logSinkContext);

	common::error::UnwindException(err, logger);
}

}
