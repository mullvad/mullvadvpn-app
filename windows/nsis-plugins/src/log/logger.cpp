#include "stdafx.h"
#include "logger.h"
#include <libcommon/error.h>
#include <libcommon/string.h>
#include <sstream>
#include <iomanip>

Utf8FileLogSink::Utf8FileLogSink(const std::wstring &file, bool append, bool flush)
	: m_flush(flush)
{
	const DWORD creationDisposition = (append ? OPEN_ALWAYS : CREATE_ALWAYS);

	m_logfile = CreateFileW(file.c_str(), GENERIC_READ | GENERIC_WRITE, FILE_SHARE_READ, nullptr,
		creationDisposition, FILE_ATTRIBUTE_NORMAL, nullptr);

	if (INVALID_HANDLE_VALUE == m_logfile)
	{
		THROW_WINDOWS_ERROR(GetLastError(), "Open/create log file");
	}

	if (append && ERROR_ALREADY_EXISTS == GetLastError())
	{
		LARGE_INTEGER offset = { 0 };

		const auto seekStatus = SetFilePointerEx(m_logfile, offset, nullptr, FILE_END);

		if (FALSE == seekStatus)
		{
			CloseHandle(m_logfile);
			THROW_WINDOWS_ERROR(GetLastError(), "Seek to end offset in existing log file");
		}
	}
}

Utf8FileLogSink::~Utf8FileLogSink()
{
	CloseHandle(m_logfile);
}

void Utf8FileLogSink::log(const std::wstring &message)
{
	auto utf8String = common::string::ToUtf8(message);
	utf8String.pop_back(); // remove the null char

	if (0 == utf8String.size())
	{
		return;
	}

	utf8String.push_back('\xd');
	utf8String.push_back('\xa');

	DWORD bytesWritten;

	WriteFile(m_logfile, utf8String.data(), utf8String.size(), &bytesWritten, nullptr);

	if (m_flush)
	{
		FlushFileBuffers(m_logfile);
	}
}

void Logger::log(const std::wstring &message)
{
	m_logsink->log(Compose(message, Timestamp()));
}

void Logger::log(const std::wstring &message, const std::vector<std::wstring> &details)
{
	const auto timestamp = this->Timestamp();

	m_logsink->log(Compose(message, timestamp));

	//
	// Write details with indentation.
	//
	for (const auto detail : details)
	{
		m_logsink->log(Compose(detail, timestamp, 4));
	}
}

// static
std::wstring Logger::Timestamp()
{
	SYSTEMTIME time;

	GetLocalTime(&time);

	std::wstringstream ss;

	ss << L'['
		<< std::right << std::setw(4) << std::setfill(L'0') << time.wYear
		<< L'-'
		<< std::right << std::setw(2) << std::setfill(L'0') << time.wMonth
		<< L'-'
		<< std::right << std::setw(2) << std::setfill(L'0') << time.wDay
		<< L' '
		<< std::right << std::setw(2) << std::setfill(L'0') << time.wHour
		<< L':'
		<< std::right << std::setw(2) << std::setfill(L'0') << time.wMinute
		<< L':'
		<< std::right << std::setw(2) << std::setfill(L'0') << time.wSecond
		<< L'.'
		<< std::right << std::setw(3) << std::setfill(L'0') << time.wMilliseconds
		<< L']';

	return ss.str();
}

//static
std::wstring Logger::Compose(const std::wstring &message, const std::wstring &timestamp, size_t indentation)
{
	std::wstringstream ss;

	ss << timestamp << L' '
		<< std::wstring(indentation, L' ')
		<< message;

	return ss.str();
}
