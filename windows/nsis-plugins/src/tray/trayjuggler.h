#pragma once

#include "iconstreams.h"
#include "trayparser.h"
#include <cstdint>
#include <vector>
#include <memory>
#include <string>
#include <functional>
#include <list>

class TrayJuggler
{
public:

	TrayJuggler(const TrayParser &parser);

	// Find record based on substring present in record's application path.
	std::shared_ptr<ICON_STREAMS_RECORD> findRecord(const std::wstring &path) const;

	enum class TraySearchGroup
	{
		Visible,
		Hidden
	};

	uint32_t getNextFreeOrdinal(TraySearchGroup searchGroup) const;

	//
	// Fix up and inject record.
	// The icon will be displayed in the rightmost position.
	//
	void injectRecord(const ICON_STREAMS_RECORD &record);

	//
	// Update and promote existing record.
	// The icon will be displayed in the rightmost position.
	//
	void promoteRecord(std::shared_ptr<ICON_STREAMS_RECORD> record);
	
	bool enumerateRecords(std::function<bool(std::shared_ptr<ICON_STREAMS_RECORD> record)> callback) const;

	// Generate a valid stream including header.
	std::vector<uint8_t> pack() const;

	static std::wstring DecodeString(const uint16_t *encoded, size_t bufferSize);

private:

	//
	// This is the original header.
	// We keep it around to be able to preserve the values for
	// the unknown fields.
	//
	ICON_STREAMS_HEADER m_header;

	std::list<std::shared_ptr<ICON_STREAMS_RECORD> > m_records;
};
