#include "stdafx.h"
#include "logger.h"
#include <libcommon/string.h>
#include <libcommon/filesystem.h>
#include <windows.h>
#include <nsis/pluginapi.h>
#include <string>
#include <vector>
#include <memory>
#include <experimental/filesystem>

Logger *g_logger = nullptr;

namespace
{

std::wstring PopString()
{
	//
	// NSIS functions popstring() and popstringn() require that you definitely size the buffer
	// before popping the string. Let's do it ourselves instead.
	//

	if (!g_stacktop || !*g_stacktop)
	{
		throw std::runtime_error("NSIS variable stack is corrupted");
	}

	stack_t *th = *g_stacktop;

	std::wstring copy(th->text);

	*g_stacktop = th->next;
	GlobalFree((HGLOBAL)th);

	return copy;
}

EXTERN_C IMAGE_DOS_HEADER __ImageBase;

void PinDll()
{
	//
	// Apparently NSIS loads and unloads the plugin module for EVERY call it makes to the plugin.
	// This makes it kind of difficult to maintain state.
	//
	// We can work around this by incrementing the module reference count.
	// When NSIS calls FreeLibrary() the reference count decrements and becomes one.
	//

	wchar_t self[MAX_PATH];

	if (0 == GetModuleFileNameW((HINSTANCE)&__ImageBase, self, _countof(self)))
	{
		throw std::runtime_error("Failed to pin plugin module");
	}

	//
	// For some reason, NSIS frees this particular DLL more times than it loads it
	// so we have to up the reference count significantly.
	//
	for (int i = 0; i < 100; ++i)
	{
		LoadLibraryW(self);
	}
}

std::vector<std::wstring> BlockToRows(const std::wstring &textBlock)
{
	//
	// This is such a hack :-(
	//
	// It only works because the tokenizer is greedy and because we don't care about
	// empty lines for this usage.
	//
	return common::string::Tokenize(textBlock, L"\r\n");
}

} // anonymous namespace

//
// Initialize
//
// Opens and maintains an open handle to the log file.
//
void __declspec(dllexport) NSISCALL Initialize
(
	HWND hwndParent,
	int string_size,
	LPTSTR variables,
	stack_t **stacktop,
	extra_parameters *extra,
	...
)
{
	EXDLL_INIT();

	try
	{
		PinDll();

		auto logfile = std::experimental::filesystem::path(common::fs::GetKnownFolderPath(
			FOLDERID_ProgramData, 0, nullptr));

		logfile.append(L"Mullvad VPN").append(L"install.log");

		g_logger = new Logger(std::make_unique<AnsiFileLogSink>(logfile));
	}
	catch (...)
	{
	}
}

//
// Log
//
// Writes a message to the log file.
//
void __declspec(dllexport) NSISCALL Log
(
	HWND hwndParent,
	int string_size,
	LPTSTR variables,
	stack_t **stacktop,
	extra_parameters *extra,
	...
)
{
	EXDLL_INIT();

	try
	{
		const auto message = PopString();

		if (g_logger != nullptr)
		{
			g_logger->log(message);
		}
	}
	catch (...)
	{
	}
}

//
// LogWithDetails
//
// Writes a message to the log file.
//
void __declspec(dllexport) NSISCALL LogWithDetails
(
	HWND hwndParent,
	int string_size,
	LPTSTR variables,
	stack_t **stacktop,
	extra_parameters *extra,
	...
)
{
	EXDLL_INIT();

	try
	{
		const auto message = PopString();
		const auto details = PopString();

		if (g_logger != nullptr)
		{
			g_logger->log(message, BlockToRows(details));
		}
	}
	catch (...)
	{
	}
}

//
// PluginLog
//
// Writes a message to the log file.
// Use from other plugins to avoid passing messages like this:
// other plugin -> NSIS -> log plugin
//
void __declspec(dllexport) NSISCALL PluginLog
(
	const std::wstring &message
)
{
	try
	{
		if (g_logger != nullptr)
		{
			g_logger->log(message);
		}
	}
	catch (...)
	{
	}
}

//
// PluginLogWithDetails
//
void __declspec(dllexport) NSISCALL PluginLogWithDetails
(
	const std::wstring &message,
	const std::vector<std::wstring> &details
)
{
	try
	{
		if (g_logger != nullptr)
		{
			g_logger->log(message, details);
		}
	}
	catch (...)
	{
	}
}
