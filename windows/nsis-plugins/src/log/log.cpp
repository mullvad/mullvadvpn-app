#include "stdafx.h"
#include "logger.h"
#include <libcommon/string.h>
#include <libcommon/filesystem.h>
#include <libcommon/registry/registry.h>
#include <libcommon/error.h>
#include <windows.h>
#include <nsis/pluginapi.h>
#include <string>
#include <vector>
#include <memory>
#include <sstream>
#include <iomanip>
#include <filesystem>
#include <mullvad-nsis.h>

Logger *g_logger = nullptr;

namespace
{

bool g_pinned = false;

std::wstring PopString()
{
	//
	// NSIS functions popstring() and popstringn() require that you definitely size the buffer
	// before popping the string. Let's do it ourselves instead.
	//

	if (!g_stacktop || !*g_stacktop)
	{
		THROW_ERROR("NSIS variable stack is corrupted");
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
	if (g_pinned)
	{
		return;
	}

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
		THROW_ERROR("Failed to pin plugin module");
	}

	//
	// For some reason, NSIS frees this particular DLL more times than it loads it
	// so we have to up the reference count significantly.
	//
	for (int i = 0; i < 100; ++i)
	{
		LoadLibraryW(self);
	}

	g_pinned = true;
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

std::wstring GetWindowsProductName()
{
	auto regkey = common::registry::Registry::OpenKey(HKEY_LOCAL_MACHINE, L"SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion",
		false, common::registry::RegistryView::Force64);

	return regkey->readString(L"ProductName");
}

std::wstring GetWindowsVersion()
{
	std::vector<uint16_t> productVersion(256);
	size_t bufferSize = productVersion.size();

	// Simply call into rust function, which will use the correct
	// syscall/Windows API to retrieve the Windows version.
	auto result = get_system_version(productVersion.data(), &bufferSize);
	if (Status::Ok != result)
	{
		THROW_ERROR("Failed to acquire Windows version");
	}

	return std::wstring(reinterpret_cast<wchar_t *>(productVersion.data()));
}

//
// FixupWindows11ProductName()
//
// Patch product name based on Windows version.
// The registry value that holds the product name seems to be deprecated in Win11.
//
std::wstring FixupWindows11ProductName(const std::wstring &productName, const std::wstring &version)
{
	const auto versionTokens = common::string::Tokenize(version, L".");

	if (versionTokens.size() < 3)
	{
		return productName;
	}

	const auto major = common::string::LexicalCast<uint32_t>(versionTokens[0]);
	const auto minor = common::string::LexicalCast<uint32_t>(versionTokens[1]);
	const auto build = common::string::LexicalCast<uint32_t>(versionTokens[2]);

	if (major != 10 || minor != 0 || build < 22000)
	{
		return productName;
	}

	auto productTokens = common::string::Tokenize(productName, L" ");

	for (auto &token : productTokens)
	{
		if (0 == token.compare(L"10"))
		{
			token = L"11";
		}
	}

	return common::string::Join(productTokens, L" ");
}

} // anonymous namespace

//
// SetLogTarget
//
// Opens and maintains an open handle to the log file.
//
enum class LogTarget
{
	LOG_INSTALL = 0,
	LOG_UNINSTALL = 1,
	LOG_VOID = 2
};

void __declspec(dllexport) NSISCALL SetLogTarget
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

		const wchar_t *logfile = nullptr;

		int target = popint();
		switch (target)
		{
			case static_cast<int>(LogTarget::LOG_INSTALL):
			{
				logfile = L"install.log";
				break;
			}
			case static_cast<int>(LogTarget::LOG_UNINSTALL):
			{
				logfile = L"uninstall.log";
				break;
			}
			case static_cast<int>(LogTarget::LOG_VOID):
			{
				delete g_logger;
				g_logger = nullptr;
				return;
			}
			default:
			{
				THROW_ERROR("Invalid log target");
			}
		}

		if (nullptr == logfile)
		{
			THROW_ERROR("Invalid log target");
		}

		auto logpath = std::filesystem::path(common::fs::GetKnownFolderPath(
			FOLDERID_ProgramData));
		logpath.append(L"Mullvad VPN");

		auto logpath_wstring = logpath.wstring();
		const wchar_t* w_path = logpath_wstring.c_str();

		if (Status::Ok != create_privileged_directory(reinterpret_cast<const uint16_t*>(w_path)))
		{
		    THROW_ERROR("Failed to create log directory");
		}

		logpath.append(logfile);

		g_logger = new Logger(std::make_unique<Utf8FileLogSink>(logpath, false));
	}
	catch (std::exception &err)
	{
		std::stringstream ss;

		ss << "Failed to set logging plugin target."
			<< std::endl
			<< err.what();

		MessageBoxA(hwndParent, ss.str().c_str(), nullptr, MB_OK);
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
// LogWindowsVersion
//
// Writes a message containing the Windows version and build, to the log file.
//
void __declspec(dllexport) NSISCALL LogWindowsVersion
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

	if (nullptr == g_logger)
	{
		return;
	}

	try
	{
		const auto productName = GetWindowsProductName();
		const auto version = GetWindowsVersion();

		std::wstringstream ss;

		ss	<< L"Windows version: "
			<< FixupWindows11ProductName(productName, version)
			<< L", "
			<< version;

		g_logger->log(ss.str());
	}
	catch (std::exception &err)
	{
		const std::vector<std::wstring> details =
		{
			common::string::ToWide(err.what())
		};

		g_logger->log(L"Windows version: Failed to determine version", details);
	}
	catch (...)
	{
		g_logger->log(L"Windows version: Failed to determine version");
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
