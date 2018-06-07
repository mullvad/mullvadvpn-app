#include "stdafx.h"
#include "serializer.h"
#include <algorithm>

namespace
{

void Append(std::vector<uint8_t> &v, const void *data, size_t length)
{
	if (0 == length)
	{
		return;
	}

	const auto oldSize = v.size();

	v.resize(oldSize + length);
	memcpy(&v[oldSize], data, length);
}

std::vector<uint8_t> PackageString(const wchar_t *str)
{
	const auto strByteLength = static_cast<uint32_t>(wcslen(str) * sizeof(wchar_t));

	std::vector<uint8_t> data;

	Append(data, &strByteLength, sizeof(strByteLength));
	Append(data, str, strByteLength);

	return data;
}

} // anonymous namespace

namespace common::serialization
{

void Serializer::operator<<(uint8_t data)
{
	append(TypeTag::Uint8, &data, sizeof(uint8_t));
}

void Serializer::operator<<(uint16_t data)
{
	append(TypeTag::Uint16, &data, sizeof(uint16_t));
}

void Serializer::operator<<(uint32_t data)
{
	append(TypeTag::Uint32, &data, sizeof(uint32_t));
}

void Serializer::operator<<(const GUID &data)
{
	append(TypeTag::Guid, &data, sizeof(GUID));
}

void Serializer::operator<<(const std::wstring &data)
{
	auto packaged = PackageString(data.c_str());

	append(TypeTag::String, &packaged[0], packaged.size());
}

void Serializer::operator<<(const wchar_t *data)
{
	auto packaged = PackageString(data);

	append(TypeTag::String, &packaged[0], packaged.size());
}

void Serializer::operator<<(const std::vector<std::wstring> &data)
{
	std::vector<uint8_t> arrayBlob;

	uint32_t count = static_cast<uint32_t>(data.size());

	Append(arrayBlob, &count, sizeof(count));

	std::for_each(data.begin(), data.end(), [&](const std::wstring &str)
	{
		auto packagedStr = PackageString(str.c_str());

		// Hack? Makes parsing a lot simpler
		arrayBlob.push_back(static_cast<uint8_t>(TypeTag::String));

		Append(arrayBlob, &packagedStr[0], packagedStr.size());
	});

	append(TypeTag::StringArray, &arrayBlob[0], arrayBlob.size());
}

const std::vector<uint8_t> &Serializer::blob() const
{
	return m_blob;
}

void Serializer::append(TypeTag type, const void *data, size_t length)
{
	const auto elementSize = sizeof(uint8_t) + length;
	const auto oldSize = m_blob.size();

	m_blob.resize(oldSize + elementSize);

	auto dest = &m_blob[oldSize];

	*dest++ = static_cast<uint8_t>(type);
	memcpy(dest, data, length);
}

}
