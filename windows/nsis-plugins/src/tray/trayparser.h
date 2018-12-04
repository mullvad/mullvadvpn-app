#pragma once

#include "iconstreams.h"
#include <cstdint>
#include <vector>
#include <memory>

class TrayParser
{
public:

	TrayParser(const std::vector<uint8_t> &blob);

	const ICON_STREAMS_HEADER &getHeader() const;

	const std::vector<ICON_STREAMS_RECORD> &getRecords() const;

private:

	ICON_STREAMS_HEADER m_header;
	std::vector<ICON_STREAMS_RECORD> m_records;
};
