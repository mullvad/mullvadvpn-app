#pragma once

#include "wfpctl/iobjectinstaller.h"

//
// A firewall rule uses one of more filters to implement a concept
// E.g. "allow lan traffic"
//

namespace rules
{

struct IFirewallRule
{
	virtual ~IFirewallRule() = 0
	{
	}

	//virtual std::wstring name() = 0;
	//virtual std::vector<std::wstring> details() = 0; // doesn't work? because there can be multiple filters each with multiple conditions

	virtual bool apply(IObjectInstaller &objectInstaller) = 0;
};

}
