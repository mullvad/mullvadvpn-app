// cli.cpp : Defines the entry point for the console application.
//

#include "stdafx.h"
#include "util.h"
#include "filterengineprovider.h"
#include "modules/imodule.h"
#include "modules/list.h"
#include "modules/monitor.h"
#include "modules/winfw.h"
#include "libcommon/string.h"
#include <iostream>
#include <conio.h>

// Should set key comparison operator to compare lowercase strings but meh.
std::map<std::wstring, std::unique_ptr<modules::IModule> > g_modules;

void OutputConsole(const std::wstring &str)
{
	std::wcout << str.c_str() << std::endl;
}

void InitializeFilterEngine()
{
	FilterEngineProvider::Instance().set(std::move(wfp::FilterEngine::DynamicSession()));
}

void InitializeModules()
{
	auto list = std::make_unique<modules::List>(OutputConsole);
	g_modules.insert(std::make_pair(common::string::Lower(list->name()), std::move(list)));

	auto monitor = std::make_unique<modules::Monitor>(OutputConsole);
	g_modules.insert(std::make_pair(common::string::Lower(monitor->name()), std::move(monitor)));

	auto winfw = std::make_unique<modules::WinFw>(OutputConsole);
	g_modules.insert(std::make_pair(common::string::Lower(winfw->name()), std::move(winfw)));
}

void ProcessHelp(const std::wstring &request)
{
	auto tokens = common::string::Tokenize(request, L" ");

	if (tokens.empty())
	{
		std::wcout << L"Unable to interpret request." << std::endl;
		return;
	}

	if (tokens.size() == 1)
	{
		PropertyList list;
		
		for (auto &module : g_modules)
		{
			list.add(common::string::Lower(module.second->name()), module.second->description());
		}

		list.add(L"help", L"List available modules.");
		list.add(L"help /module/", L"Show module specific help.");
		list.add(L"reset", L"Reset the Filter Engine session.");
		list.add(L"quit", L"Exit the application.");

		PrettyPrintOptions options;

		options.indent = 0;
		options.useSeparator = false;

		PrettyPrintProperties(OutputConsole, options, list);

		return;
	}

	if (tokens.size() != 2)
	{
		std::wcout << L"Unable to interpret request." << std::endl;
		return;
	}

	auto wanted = common::string::Lower(tokens[1]);
	auto found = g_modules.find(wanted);

	if (found == g_modules.end())
	{
		std::wcout << L"No such module: " << wanted << L"." << std::endl;
		return;
	}

	auto moduleCommands = found->second->commands();

	PrettyPrintOptions options;

	options.indent = 0;
	options.useSeparator = false;

	PrettyPrintProperties(OutputConsole, options, moduleCommands);
}

void ProcessRequest(const std::wstring &request)
{
	auto tokens = common::string::Tokenize(request, L" ");

	if (tokens.empty())
	{
		std::wcout << L"Unable to interpret request." << std::endl;
		return;
	}

	auto wanted = common::string::Lower(tokens[0]);
	auto found = g_modules.find(wanted);

	if (found == g_modules.end())
	{
		std::wcout << L"No such module: " << wanted << L"." << std::endl;
		return;
	}

	tokens.erase(tokens.begin());

	found->second->handleRequest(tokens);
}

int main(int, wchar_t **)
{
	InitializeFilterEngine();
	InitializeModules();

	for (;;)
	{
		std::wcout << L"wfp> ";

		std::wstring request;
		std::getline(std::wcin, request);

		if (0 == _wcsicmp(request.c_str(), L"quit"))
		{
			break;
		}

		if (0 == _wcsicmp(request.c_str(), L"reset"))
		{
			InitializeFilterEngine();
			std::wcout << std::endl;

			continue;
		}

		if (0 == _wcsnicmp(request.c_str(), L"help", 4))
		{
			ProcessHelp(request);
			std::wcout << std::endl;

			continue;
		}

		try
		{
			ProcessRequest(request);
		}
		catch (std::exception &error)
		{
			std::cout << "Error: " << error.what() << std::endl;
		}
		catch (...)
		{
			std::cout << "Unknown error, caught exception!" << std::endl;
		}

		std::wcout << std::endl;
	}

	return 0;
}
