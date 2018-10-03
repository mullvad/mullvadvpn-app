#pragma once

#include "irecoverysink.h"
#include "clientsinkinfo.h"
#include <vector>
#include <cstdint>
#include <mutex>

class RecoverySink : public IRecoverySink
{
public:

	RecoverySink(const RecoverySinkInfo &target);

	void setTarget(const RecoverySinkInfo &target);

	void preserveSnaps(Protocol protocol, const std::vector<InterfaceSnap> &snaps) override;

	std::vector<uint8_t> recoveryData() const;

private:

	std::mutex m_targetMutex;
	RecoverySinkInfo m_target;

	mutable std::mutex m_dataMutex;
	std::vector<InterfaceSnap> m_v4Snaps;
	std::vector<InterfaceSnap> m_v6Snaps;
	std::vector<uint8_t> m_recoveryData;
};
