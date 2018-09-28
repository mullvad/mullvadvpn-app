#pragma once

#include <cstdint>

struct ILogSink
{
	virtual ~ILogSink() = 0
	{
	}

	virtual void error(const char *msg, const char **details, uint32_t numDetails) = 0;

	virtual void error(const char *msg)
	{
		error(msg, nullptr, 0);
	}

	virtual void info(const char *msg, const char **details, uint32_t numDetails) = 0;

	virtual void info(const char *msg)
	{
		info(msg, nullptr, 0);
	}
};
