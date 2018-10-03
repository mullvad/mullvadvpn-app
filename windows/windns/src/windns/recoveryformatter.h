#pragma once

#include "interfacesnap.h"
#include <vector>
#include <cstdint>

class RecoveryFormatter
{
public:

	RecoveryFormatter() = delete;

	static std::vector<uint8_t> Pack(const std::vector<InterfaceSnap> &v4Snaps,
		const std::vector<InterfaceSnap> &v6Snaps);

	struct Unpacked
	{
		std::vector<InterfaceSnap> v4Snaps;
		std::vector<InterfaceSnap> v6Snaps;
	};

	static Unpacked Unpack(const uint8_t *data, uint32_t dataSize);
	static Unpacked Unpack(const std::vector<uint8_t> &data);
};
