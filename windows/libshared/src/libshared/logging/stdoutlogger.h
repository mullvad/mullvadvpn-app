#pragma once

#include "logsink.h"

namespace shared::logging
{

void __stdcall StdoutLogger(MULLVAD_LOG_LEVEL level, const char *msg, void *context);

}
