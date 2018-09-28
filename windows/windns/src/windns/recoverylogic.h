#pragma once

#include "recoveryformatter.h"
#include "ilogsink.h"

class RecoveryLogic
{
public:

	RecoveryLogic() = delete;

	static void RestoreInterfaces(const RecoveryFormatter::Unpacked &data,
		ILogSink *logSink, uint32_t timeout = 0);
};
