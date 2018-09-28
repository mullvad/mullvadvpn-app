#include "stdafx.h"
#include "recoverysink.h"
#include "recoveryformatter.h"
#include <stdexcept>

RecoverySink::RecoverySink(const RecoverySinkInfo &target)
	: m_target(target)
{
}

void RecoverySink::setTarget(const RecoverySinkInfo &target)
{
	std::scoped_lock<std::mutex> lock(m_targetMutex);

	m_target = target;
}

void RecoverySink::preserveSnaps(Protocol protocol, const std::vector<InterfaceSnap> &snaps)
{
	std::scoped_lock<std::mutex> dataLock(m_dataMutex);

	switch (protocol)
	{
		case Protocol::IPv4:
		{
			m_v4Snaps = snaps;
			break;
		}
		case Protocol::IPv6:
		{
			m_v6Snaps = snaps;
			break;
		}
		default:
		{
			throw std::runtime_error("Missing case handler");
		}
	}

	m_recoveryData = RecoveryFormatter::Pack(m_v4Snaps, m_v6Snaps);

	std::scoped_lock<std::mutex> lock(m_targetMutex);

	m_target.sink(&m_recoveryData[0], static_cast<uint32_t>(m_recoveryData.size()), m_target.context);
}

std::vector<uint8_t> RecoverySink::recoveryData() const
{
	std::vector<uint8_t> copy;

	{
		std::scoped_lock<std::mutex> dataLock(m_dataMutex);

		copy = m_recoveryData;
	}

	return copy;
}
