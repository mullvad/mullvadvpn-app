#pragma once

#include "logsink.h"
#include <stdexcept>

namespace shared::logging
{

void UnwindAndLog(MullvadLogSink logSink, void *logSinkContext, const std::exception &err);

}
