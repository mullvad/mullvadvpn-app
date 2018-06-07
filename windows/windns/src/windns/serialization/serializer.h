#pragma once

#include "typetag.h"
#include <vector>
#include <string>
#include <cstdint>
#include <guiddef.h>

namespace common::serialization
{

class Serializer
{
public:

	void operator<<(uint8_t data);
	void operator<<(uint16_t data);
	void operator<<(uint32_t data);
	void operator<<(const GUID &data);
	void operator<<(const std::wstring &data);
	void operator<<(const wchar_t *data);
	void operator<<(const std::vector<std::wstring> &data);

	const std::vector<uint8_t> &blob() const;

private:

	std::vector<uint8_t> m_blob;

	void append(TypeTag type, const void *data, size_t length);
};

}
