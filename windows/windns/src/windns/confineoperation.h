#pragma once

#include <libcommon/logging/ilogsink.h>
#include <functional>
#include <vector>
#include <string>
#include <cstdint>

bool ConfineOperation
(
	const char *literalOperation,
	std::shared_ptr<common::logging::ILogSink> logSink,
	std::function<void()> operation
);
