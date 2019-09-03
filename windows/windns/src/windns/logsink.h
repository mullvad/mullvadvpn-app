#pragma once

#include "ilogsink.h"
#include <mutex>

class LogSink : public ILogSink
{
public:

	LogSink(const LogSinkInfo &target);

	void setTarget(const LogSinkInfo &target);

	void error(const char *msg, const char **details, uint32_t numDetails) override;
	void info(const char *msg, const char **details, uint32_t numDetails) override;

private:

	std::mutex m_targetMutex;
	LogSinkInfo m_target;
};
