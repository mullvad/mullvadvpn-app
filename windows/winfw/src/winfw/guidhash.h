#pragma once

#include <cstdint>
#include <utility>
#include <guiddef.h>

// Specialize std::hash
namespace std
{

template<>
struct hash<GUID>
{
	size_t operator()(const GUID &guid) const noexcept
	{
		static_assert(sizeof(GUID) == (2 * sizeof(uint64_t)));

		// MOV on x86 supports non-aligned access.
		auto data = reinterpret_cast<const uint64_t *>(&guid);

		return hash<uint64_t>()(data[0] ^ data[1]);
	}
};

}
