#pragma once

#include "types.h"
#include <vector>
#include <string>

//
// Provide the array of name servers that we enforce on all adapters.
//
struct INameServerSource
{
	virtual ~INameServerSource() = 0
	{
	}

	virtual std::vector<std::wstring> getNameServers(Protocol protocol) const = 0;

	//
	// Get notified if the servers array is updated.
	//
	virtual void subscribe(HANDLE eventHandle) = 0;
	virtual void unsubscribe(HANDLE eventHandle) = 0;
};
