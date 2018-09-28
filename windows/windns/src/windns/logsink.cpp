#include "stdafx.h"
#include "logsink.h"

LogSink::LogSink(const LogSinkInfo &target)
	: m_target(target)
{
}

void LogSink::setTarget(const LogSinkInfo &target)
{
	std::scoped_lock<std::mutex> lock(m_targetMutex);

	m_target = target;
}

void LogSink::error(const char *msg, const char **details, uint32_t numDetails)
{
	std::scoped_lock<std::mutex> lock(m_targetMutex);

	if (nullptr != m_target.sink)
	{
		m_target.sink(WINDNS_LOG_CATEGORY_ERROR, msg, details, numDetails, m_target.context);
	}
}

void LogSink::info(const char *msg, const char **details, uint32_t numDetails)
{
	std::scoped_lock<std::mutex> lock(m_targetMutex);

	if (nullptr != m_target.sink)
	{
		m_target.sink(WINDNS_LOG_CATEGORY_INFO, msg, details, numDetails, m_target.context);
	}
}
