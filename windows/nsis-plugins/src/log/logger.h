#pragma once

#include <string>
#include <vector>
#include <memory>
#include <windows.h>

struct ILogSink
{
	virtual ~ILogSink() = 0
	{
	}

	virtual void log(const std::wstring &message) = 0;
};

class VoidLogSink : public ILogSink
{
public:

	void log(const std::wstring &message) override {}
};

class Utf8FileLogSink : public ILogSink
{
public:

	Utf8FileLogSink(const std::wstring &file, bool append = true, bool flush = false);
	~Utf8FileLogSink();

	Utf8FileLogSink(const Utf8FileLogSink &) = delete;
	Utf8FileLogSink &operator=(const Utf8FileLogSink &) = delete;

	void log(const std::wstring &message) override;

private:

	HANDLE m_logfile = INVALID_HANDLE_VALUE;
	bool m_flush;
};

class Logger
{
public:

	Logger(std::unique_ptr<ILogSink> &&logsink)
		: m_logsink(std::move(logsink))
	{
	}

	Logger(const Logger &) = delete;
	Logger &operator=(const Logger &) = delete;

	void log(const std::wstring &message);
	void log(const std::wstring &message, const std::vector<std::wstring> &details);

private:

	std::unique_ptr<ILogSink> m_logsink;

	static std::wstring Timestamp();

	static std::wstring Compose(const std::wstring &message, const std::wstring &timestamp,
		size_t indentation = 0);
};
