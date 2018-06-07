#pragma once

#include "typetag.h"
#include <vector>
#include <string>
#include <cstdint>
#include <guiddef.h>

namespace common::serialization
{

class Deserializer
{
public:

	Deserializer(const uint8_t *blob, size_t size);

	void operator>>(uint8_t &data);
	void operator>>(uint16_t &data);
	void operator>>(uint32_t &data);
	void operator>>(GUID &data);
	void operator>>(std::wstring &data);
	void operator>>(std::vector<std::wstring> &data);

private:

	std::vector<uint8_t> m_blob;
	size_t m_offset;

	void validateType(TypeTag type);
	void read(void *data, size_t length);
};

}
