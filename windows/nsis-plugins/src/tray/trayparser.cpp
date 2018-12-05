#include "stdafx.h"
#include "trayparser.h"
#include <stdexcept>
#include <algorithm>

TrayParser::TrayParser(const std::vector<uint8_t> &blob)
{
	if (blob.size() < sizeof(ICON_STREAMS_HEADER))
	{
		throw std::runtime_error("Invalid icon streams header - truncated");
	}

	auto header = reinterpret_cast<const ICON_STREAMS_HEADER *>(&blob[0]);

	if (header->HeaderSize != sizeof(ICON_STREAMS_HEADER))
	{
		throw std::runtime_error("Invalid icon streams header - size mismatch");
	}

	memcpy(&m_header, header, sizeof(ICON_STREAMS_HEADER));

	if (0 == header->NumberRecords)
	{
		return;
	}

	//
	// At least one record.
	//

	if (blob.size() < sizeof(ICON_STREAMS_HEADER) + sizeof(ICON_STREAMS_RECORD))
	{
		throw std::runtime_error("Invalid icon streams - truncated");
	}

	const auto lastValidRecordOffset = blob.size() - sizeof(ICON_STREAMS_RECORD);

	if (header->OffsetFirstRecord < header->HeaderSize
		|| header->OffsetFirstRecord > lastValidRecordOffset)
	{
		throw std::runtime_error("Invalid icon streams header - record offset");
	}

	const auto estimatedSize = header->HeaderSize
		+ (header->OffsetFirstRecord - header->HeaderSize)
		+ (header->NumberRecords * sizeof(ICON_STREAMS_RECORD));

	if (blob.size() != estimatedSize)
	{
		throw std::runtime_error("Invalid icon streams - size mismatch");
	}

	//
	// Size checks out.
	//

	m_records.reserve(header->NumberRecords);

	auto begin = reinterpret_cast<const ICON_STREAMS_RECORD *>(&blob[0] + header->OffsetFirstRecord);
	auto end = reinterpret_cast<const ICON_STREAMS_RECORD *>(&blob[0] + blob.size());

	std::copy(begin, end, std::back_inserter(m_records));
}

const ICON_STREAMS_HEADER &TrayParser::getHeader() const
{
	return m_header;
}

const std::vector<ICON_STREAMS_RECORD> &TrayParser::getRecords() const
{
	return m_records;
}
