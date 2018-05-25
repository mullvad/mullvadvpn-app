#pragma once

#include "cli/propertylist.h"
#include <string>
#include <vector>

namespace modules
{

struct IModule
{
	virtual std::wstring name() = 0;
	virtual std::wstring description() = 0;
	virtual PropertyList commands() = 0;

	virtual void handleRequest(const std::vector<std::wstring> &request) = 0;
};

}
