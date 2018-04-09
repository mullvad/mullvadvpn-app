#pragma once

#include <string>
#include <vector>

namespace commands
{

struct ICommand
{
	virtual std::wstring name() = 0;
	virtual std::wstring description() = 0;

	virtual void handleRequest(const std::vector<std::wstring> &arguments) = 0;
};

}
