#pragma once

namespace common::serialization
{

enum class TypeTag
{
	Uint8,
	Uint16,
	Uint32,
	Guid,		// data = binary 16 bytes
	String,		// data = [uint32: byte length], [UCS-2 string], [NOT zero terminated]
	StringArray	// data = [uint32: count], count * String
};

//
// Each entry is serialized as:
// [uint8: type], [data]
//

}
