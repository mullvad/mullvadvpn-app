#include "stdafx.h"
#include "logger.h"
#include <libcommon/string.h>
#include <libcommon/filesystem.h>
#include <libcommon/registry/registry.h>
#include <libcommon/filesystem.h>
#include <libcommon/error.h>
#include <windows.h>
#include <nsis/pluginapi.h>
#include <string>
#include <vector>
#include <memory>
#include <sstream>
#include <iomanip>
#include <filesystem>

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
	common::fs::ScopedNativeFileSystem nativeFileSystem;

	const auto systemDir = common::fs::GetKnownFolderPath(FOLDERID_System, 0, nullptr);
	const auto systemModule = std::filesystem::path(systemDir).append(L"ntoskrnl.exe");

	DWORD dummy;

	const auto versionSize = GetFileVersionInfoSizeW(systemModule.c_str(), &dummy);

	if (0 == versionSize)
	{
		THROW_WINDOWS_ERROR(GetLastError(), "GetFileVersionInfoSizeW");
	}

	std::vector<uint8_t> buf(versionSize);

	auto status = GetFileVersionInfoW(systemModule.c_str(), 0, static_cast<DWORD>(buf.size()), &buf[0]);

	if (FALSE == status)
	{
		THROW_WINDOWS_ERROR(GetLastError(), "GetFileVersionInfoW");
	}

	//
	// Get the translation table.
	// This is required to build the path to the value we're actually after.
	//

	struct LANGANDCODEPAGE
	{
		WORD wLanguage;
		WORD wCodePage;
	}
	*translations = nullptr;

	UINT translationsSize = 0;

	status = VerQueryValueW(&buf[0], L"\\VarFileInfo\\Translation", reinterpret_cast<LPVOID *>(&translations), &translationsSize);

	if (FALSE == status)
	{
		THROW_WINDOWS_ERROR(GetLastError(), "VerQueryValueW");
	}

	if (translationsSize < sizeof(LANGANDCODEPAGE))
	{
		THROW_ERROR("Invalid VERSION_INFO translation table");
	}

	//
	// Use primary translation.
	//

	std::wstringstream ss;

	ss << L"\\StringFileInfo\\"
		<< std::setw(4) << std::setfill(L'0') << std::hex
		<< translations[0].wLanguage
		<< std::setw(4) << std::setfill(L'0') << std::hex
		<< translations[0].wCodePage
		<< L"\\ProductVersion";

	const auto productVersionName = ss.str();

	void *productVersion = nullptr;
	UINT productVersionSize = 0;

	status = VerQueryValueW(&buf[0], productVersionName.c_str(), &productVersion, &productVersionSize);

	if (FALSE == status)
	{
		THROW_WINDOWS_ERROR(GetLastError(), "VerQueryValueW");
	}

	// Size returned is the length in characters.
	std::wstring version(reinterpret_cast<const wchar_t *>(productVersion), productVersionSize);

	// Chop off trailing terminators.
	while ((false == version.empty()) && (*version.rbegin() == L'\0'))
	{
		version.resize(version.size() - 1);
	}

	if (version.empty())
	{
		THROW_ERROR("Invalid version information");
	}

	return version;
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
			FOLDERID_ProgramData, 0, nullptr));
		logpath.append(L"Mullvad VPN");

		if (FALSE == CreateDirectoryW(logpath.c_str(), nullptr))
		{
			if (ERROR_ALREADY_EXISTS != GetLastError())
			{
				std::wstringstream ss;

				ss << L"Cannot create folder: "
					<< L"\""
					<< logpath
					<< L"\"";

				THROW_ERROR(common::string::ToAnsi(ss.str()).c_str());
			}
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
			<< productName
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
