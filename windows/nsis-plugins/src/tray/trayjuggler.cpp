#include "stdafx.h"
#include "trayjuggler.h"
#include <algorithm>

namespace
{

template<typename T>
void AppendBuffer(std::vector<uint8_t> &buffer, const T &t)
{
	const auto prevSize = buffer.size();

	buffer.resize(prevSize + sizeof(T));

	memcpy(&buffer[prevSize], &t, sizeof(T));
}

inline wchar_t rot13(wchar_t c)
{
	if (c >= L'A' && c <= L'Z')
	{
		auto temp = (wchar_t)(c + 13);
		return (temp <= L'Z' ? temp : (wchar_t)(L'A' + (temp - L'Z' - 1)));
	}
	else if (c >= L'a' && c <= L'z')
	{
		auto temp = (wchar_t)(c + 13);
		return (temp <= L'z' ? temp : (wchar_t)(L'a' + (temp - L'z' - 1)));
	}

	return c;
}

} // anonymous namespace

TrayJuggler::TrayJuggler(const TrayParser &parser)
{
	m_header = parser.getHeader();

	for (const auto &record : parser.getRecords())
	{
		m_records.push_back(std::make_shared<ICON_STREAMS_RECORD>(record));
	}
}

std::shared_ptr<ICON_STREAMS_RECORD> TrayJuggler::findRecord(const std::wstring &path) const
{
	for (const auto &record : m_records)
	{
		const auto appPath = DecodeString(record->ApplicationPath, sizeof(record->ApplicationPath));

		if (nullptr != wcsstr(appPath.c_str(), path.c_str()))
		{
			return record;
		}
	}

	return std::shared_ptr<ICON_STREAMS_RECORD>();
}

uint32_t TrayJuggler::getNextFreeOrdinal(TraySearchGroup searchGroup) const
{
	uint32_t highestOrdinal = 0;

	bool noMatchingRecord = true;

	enumerateRecords([&](std::shared_ptr<ICON_STREAMS_RECORD> record)
	{
		if ((TraySearchGroup::Visible == searchGroup && ICON_STREAMS_VISIBILITY::SHOW_ICON_AND_NOTIFICATIONS == record->Visibility)
			|| (TraySearchGroup::Hidden == searchGroup) && ICON_STREAMS_VISIBILITY::SHOW_ICON_AND_NOTIFICATIONS != record->Visibility)
		{
			noMatchingRecord = false;

			if (record->Ordinal > highestOrdinal)
			{
				highestOrdinal = record->Ordinal;
			}
		}

		return true;
	});

	return (noMatchingRecord ? 0 : highestOrdinal + 1);
}


void TrayJuggler::injectRecord(const ICON_STREAMS_RECORD &record)
{
	SYSTEMTIME time;

	GetSystemTime(&time);

	auto newRecord = std::make_shared<ICON_STREAMS_RECORD>(record);

	{
		newRecord->Visibility = ICON_STREAMS_VISIBILITY::SHOW_ICON_AND_NOTIFICATIONS;

		newRecord->YearCreated = time.wYear;
		newRecord->MonthCreated = time.wMonth;

		// TODO: Meaning of this bool?
		newRecord->u7 = 0;

		newRecord->ImagelistId = 0xFFFFFFFF;

		FILETIME fileTime;

		SystemTimeToFileTime(&time, &fileTime);

		newRecord->Time1 = fileTime;

		newRecord->Time2.dwHighDateTime = 0;
		newRecord->Time2.dwLowDateTime = 0;

		newRecord->Ordinal = getNextFreeOrdinal(TraySearchGroup::Visible);
	}

	//
	// By default, the first few records will always be Microsoft icons such as
	// battery, networking, sounds.
	//
	// These have a visibility of SHOW_ICON_AND_NOTIFICATIONS.
	//
	// There can be other icons with visibility SHOW_ICON_AND_NOTIFICATIONS towards
	// the end of the array. But these icons typically don't have a valid ImagelistId.
	//
	// In the end, the ordering of records doesn't seem to matter.
	//
	// Insert new record at the end.
	//

	m_records.push_back(newRecord);
}

void TrayJuggler::promoteRecord(std::shared_ptr<ICON_STREAMS_RECORD> record)
{
	if (ICON_STREAMS_VISIBILITY::SHOW_ICON_AND_NOTIFICATIONS == record->Visibility)
	{
		// Abort if the icon is already visible.
		return;
	}

	record->Visibility = ICON_STREAMS_VISIBILITY::SHOW_ICON_AND_NOTIFICATIONS;
	record->Ordinal = getNextFreeOrdinal(TraySearchGroup::Visible);

	SYSTEMTIME time;

	GetSystemTime(&time);

	FILETIME fileTime;

	SystemTimeToFileTime(&time, &fileTime);

	record->Time1 = fileTime;
}

bool TrayJuggler::enumerateRecords(std::function<bool(std::shared_ptr<ICON_STREAMS_RECORD> record)> callback) const
{
	for (auto record : m_records)
	{
		if (false == callback(record))
		{
			return false;
		}
	}

	return true;
}

std::vector<uint8_t> TrayJuggler::pack() const
{
	std::vector<uint8_t> blob;

	const auto requiredSize = (m_records.size() * sizeof(ICON_STREAMS_RECORD))
		+ sizeof(ICON_STREAMS_HEADER);

	blob.reserve(requiredSize);

	// Use original header as a template.
	ICON_STREAMS_HEADER header = m_header;

	header.HeaderSize = sizeof(header);
	header.NumberRecords = static_cast<uint32_t>(m_records.size());
	header.OffsetFirstRecord = sizeof(header);

	AppendBuffer(blob, header);

	for (const auto record : m_records)
	{
		AppendBuffer(blob, *record);
	}

	return blob;
}

//static
std::wstring TrayJuggler::DecodeString(const uint16_t *encoded, size_t bufferSize)
{
	const auto numCharacters = bufferSize / sizeof(uint16_t);

	std::wstring decoded;

	decoded.reserve(numCharacters);

	const auto end = encoded + numCharacters;

	for (; encoded != end; ++encoded)
	{
		auto source = (wchar_t)*encoded;

		if (L'\0' == source)
		{
			break;
		}

		decoded.push_back(rot13(source));
	}

	return decoded;
}
