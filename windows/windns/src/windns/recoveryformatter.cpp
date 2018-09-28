#include "stdafx.h"
#include <libcommon/serialization/serializer.h>
#include <libcommon/serialization/deserializer.h>
#include "recoveryformatter.h"
#include <stdexcept>

namespace
{

uint32_t RF_MAGIC = 0x21534E44;		// stores as 'DNS!'
uint32_t RF_VERSION = 0x01;

} // anonymous namespace

//static
std::vector<uint8_t> RecoveryFormatter::Pack(const std::vector<InterfaceSnap> &v4Snaps,
	const std::vector<InterfaceSnap> &v6Snaps)
{
	common::serialization::Serializer s;

	//
	// Format of binary blob
	//
	// u32		tag
	// u32		version
	// u32		number of ipv4 snaps
	// []		ipv4 snaps
	// u32		number of ipv6 snaps
	// []		ipv6 snaps
	//

	s << RF_MAGIC;
	s << RF_VERSION;

	s << static_cast<uint32_t>(v4Snaps.size());

	for (const auto &snap : v4Snaps)
	{
		snap.serialize(s);
	}

	s << static_cast<uint32_t>(v6Snaps.size());

	for (const auto &snap : v6Snaps)
	{
		snap.serialize(s);
	}

	return s.blob();
}

//static
RecoveryFormatter::Unpacked RecoveryFormatter::Unpack(const uint8_t *data, uint32_t dataSize)
{
	common::serialization::Deserializer d(data, dataSize);

	if (RF_MAGIC != d.decode<uint32_t>()
		|| RF_VERSION != d.decode<uint32_t>())
	{
		throw std::runtime_error("Invalid header in recovery data");
	}

	Unpacked unpacked;

	auto numV4Snaps = d.decode<uint32_t>();

	for (; 0 != numV4Snaps; --numV4Snaps)
	{
		// Invoke deserializing ctor on InterfaceSnap.
		unpacked.v4Snaps.emplace_back(d);
	}

	auto numV6Snaps = d.decode<uint32_t>();

	for (; 0 != numV6Snaps; --numV6Snaps)
	{
		// Invoke deserializing ctor on InterfaceSnap.
		unpacked.v6Snaps.emplace_back(d);
	}

	return unpacked;
}

//static
RecoveryFormatter::Unpacked RecoveryFormatter::Unpack(const std::vector<uint8_t> &data)
{
	return RecoveryFormatter::Unpack(&data[0], static_cast<uint32_t>(data.size()));
}
