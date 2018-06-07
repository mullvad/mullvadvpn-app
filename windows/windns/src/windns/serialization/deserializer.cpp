#include "stdafx.h"
#include "deserializer.h"
#include <stdexcept>

namespace common::serialization
{

Deserializer::Deserializer(const uint8_t *blob, size_t size)
	: m_blob(blob, blob + size)
	, m_offset(0)
{
}

void Deserializer::operator>>(uint8_t &data)
{
	validateType(TypeTag::Uint8);

	read(&data, sizeof(data));
}

void Deserializer::operator>>(uint16_t &data)
{
	validateType(TypeTag::Uint16);

	read(&data, sizeof(data));
}

void Deserializer::operator>>(uint32_t &data)
{
	validateType(TypeTag::Uint32);

	read(&data, sizeof(data));
}

void Deserializer::operator>>(GUID &data)
{
	validateType(TypeTag::Guid);

	read(&data, sizeof(data));
}

void Deserializer::operator>>(std::wstring &data)
{
	validateType(TypeTag::String);

	uint32_t strByteLength;

	read(&strByteLength, sizeof(strByteLength));

	if (0 == strByteLength)
	{
		data.clear();
		return;
	}

	std::vector<uint8_t> raw(strByteLength);

	read(&raw[0], strByteLength);

	data = std::wstring
	(
		reinterpret_cast<wchar_t *>(&raw[0]),
		reinterpret_cast<wchar_t *>(&raw[0]) + (strByteLength / sizeof(wchar_t))
	);
}

void Deserializer::operator>>(std::vector<std::wstring> &data)
{
	validateType(TypeTag::StringArray);

	uint32_t elements;

	read(&elements, sizeof(elements));

	data.clear();

	for (uint32_t i = 0; i < elements; ++i)
	{
		data.emplace_back(std::wstring());
		*this >> *data.rbegin();
	}
}

void Deserializer::validateType(TypeTag type)
{
	uint8_t readType;

	read(&readType, sizeof(uint8_t));

	if (readType != static_cast<uint8_t>(type))
	{
		throw std::runtime_error("Unexpected data type in stream");
	}
}

void Deserializer::read(void *data, size_t length)
{
	if (m_offset + length > m_blob.size())
	{
		throw std::runtime_error("Read probe passed end of stream");
	}

	memcpy(data, &m_blob[m_offset], length);

	m_offset += length;
}

}
