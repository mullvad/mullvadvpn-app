#pragma once

#include "types.h"
#include "interfacesnap.h"
#include <vector>

struct IRecoverySink
{
	virtual ~IRecoverySink() = 0
	{
	}

	virtual void preserveSnaps(Protocol protocol, const std::vector<InterfaceSnap> &snaps) = 0;
};
