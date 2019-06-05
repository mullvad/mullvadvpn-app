#include "stdafx.h"
#include "adapters.h"
#include "libcommon/error.h"
#include <sstream>
#include <stdexcept>

const IP_ADAPTER_ADDRESSES *Adapters::next() const
{
	if (nullptr == m_currentEntry)
	{
		return nullptr;
	}

	auto entry = m_currentEntry;
	m_currentEntry = m_currentEntry->Next;

	return entry;
}

Adapters::Adapters(DWORD family, DWORD flags)
{
	std::vector<uint8_t> buffer;

	static const size_t MSDN_RECOMMENDED_STARTING_BUFFER_SIZE = 1024 * 15;
	buffer.resize(MSDN_RECOMMENDED_STARTING_BUFFER_SIZE);

	ULONG bufferSize = static_cast<ULONG>(buffer.size());
	auto bufferPointer = reinterpret_cast<IP_ADAPTER_ADDRESSES *>(&buffer[0]);

	//
	// Acquire interfaces.
	//

	for (;;)
	{
		const auto status = GetAdaptersAddresses(family, flags, nullptr, bufferPointer, &bufferSize);

		if (ERROR_SUCCESS == status)
		{
			break;
		}

		if (ERROR_NO_DATA == status)
		{
			m_buffer.clear();
			m_currentEntry = nullptr;

			return;
		}

		THROW_UNLESS(ERROR_BUFFER_OVERFLOW, status, "Probe required buffer size for GetAdaptersAddresses");

		buffer.resize(bufferSize);
		bufferPointer = reinterpret_cast<IP_ADAPTER_ADDRESSES *>(&buffer[0]);
	}

	//
	// Verify structure compatibility.
	// The structure has been extended many times.
	//

	const auto systemSize = bufferPointer->Length;
	const auto codeSize = sizeof(IP_ADAPTER_ADDRESSES);

	if (systemSize < codeSize)
	{
		std::stringstream ss;

		ss << "Expecting IP_ADAPTER_ADDRESSES to have size " << codeSize << " bytes. "
			<< "Found structure with size " << systemSize << " bytes.";

		throw std::runtime_error(ss.str());
	}

	//
	// Initialize members.
	//

	m_buffer = std::move(buffer);
	reset();
}
