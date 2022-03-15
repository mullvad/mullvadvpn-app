#include "stdafx.h"
#include "cleaningops.h"
#include <libcommon/filesystem.h>
#include <libcommon/fileenumerator.h>
#include <libcommon/string.h>
#include <libcommon/error.h>
#include <libcommon/memory.h>
#include <libcommon/security.h>
#include <libcommon/process/process.h>
#include <optional>
#include <filesystem>
#include <utility>
#include <functional>
#include <processthreadsapi.h>

namespace
{

//
// Returns range in lhs that is also present in rhs.
// Equalence/equivalence determined by 'comp'.
//
// Returns pair<lhsBegin, lhsBegin> if there is no mirrored range.
//
template<typename ForwardIterator>
std::pair<ForwardIterator, ForwardIterator>
mirrored_range
(
	ForwardIterator lhsBegin, ForwardIterator lhsEnd,
	ForwardIterator rhsBegin, ForwardIterator rhsEnd,
	std::function<bool(const typename ForwardIterator::value_type &, const typename ForwardIterator::value_type &)> comp
)
{
	ForwardIterator begin = lhsBegin;

	while (lhsBegin != lhsEnd
		&& rhsBegin != rhsEnd)
	{
		if (false == comp(*lhsBegin, *rhsBegin))
		{
			break;
		}

		++lhsBegin;
		++rhsBegin;
	}

	return std::make_pair(begin, lhsBegin);
}

template<typename IterType>
std::wstring ConstructUserPath(const std::wstring &users, const std::wstring &user,
	const std::pair<IterType, IterType> &tokens)
{
	auto path = std::filesystem::path(users);

	path.append(user);

	std::for_each(tokens.first, tokens.second, [&](const std::wstring &token)
	{
		path.append(token);
	});

	return path;
}

std::wstring GetSystemUserLocalAppData()
{
	common::security::AdjustCurrentProcessTokenPrivilege(L"SeDebugPrivilege");

	common::memory::ScopeDestructor sd;

	sd += []
	{
		common::security::AdjustCurrentProcessTokenPrivilege(L"SeDebugPrivilege", false);
	};

	auto systemDir = common::fs::GetKnownFolderPath(FOLDERID_System);
	auto lsassPath = std::filesystem::path(systemDir).append(L"lsass.exe");
	auto lsassPid = common::process::GetProcessIdFromName(lsassPath);

	auto processHandle = OpenProcess(PROCESS_QUERY_INFORMATION, FALSE, lsassPid);

	if (nullptr == processHandle)
	{
		THROW_ERROR("Failed to access the \"LSASS\" process");
	}

	HANDLE processToken;

	auto status = OpenProcessToken(processHandle, TOKEN_READ | TOKEN_IMPERSONATE | TOKEN_DUPLICATE, &processToken);

	CloseHandle(processHandle);

	if (FALSE == status)
	{
		THROW_ERROR("Failed to acquire process token for the \"LSASS\" process");
	}

	sd += [&]()
	{
		CloseHandle(processToken);
	};

	return common::fs::GetKnownFolderPath(FOLDERID_LocalAppData, KF_FLAG_DEFAULT, processToken);
}

std::filesystem::path GetSystemCacheDirectory()
{
	const auto programData = common::fs::GetKnownFolderPath(FOLDERID_ProgramData);
	return std::filesystem::path(programData).append(L"Mullvad VPN").append(L"cache");
}

template <class It>
size_t EqualTokensCount(It lhsBegin, It lhsEnd, It rhsBegin, It rhsEnd)
{
	auto mirror = mirrored_range
	(
		lhsBegin, lhsEnd, rhsBegin, rhsEnd,
		[](const std::wstring &lhs, const std::wstring &rhs)
		{
			return 0 == _wcsicmp(lhs.c_str(), rhs.c_str());
		}
	);

	return static_cast<size_t>(std::distance(mirror.first, mirror.second));
}

} // anonymous namespace

namespace cleaningops
{

//
// Migrate cache for versions <= 2020.8-beta2.
//
void MigrateCacheServiceUser()
{
	const auto newCacheDir = GetSystemCacheDirectory();
	common::fs::Mkdir(newCacheDir);

	const auto localAppData = GetSystemUserLocalAppData();
	const auto oldCacheDir = std::filesystem::path(localAppData).append(L"Mullvad VPN");

	common::fs::ScopedNativeFileSystem nativeFileSystem;

	common::security::AddAdminToObjectDacl(oldCacheDir, SE_FILE_OBJECT);

	{
		common::fs::FileEnumerator files(oldCacheDir);

		auto notNamedSet = std::make_unique<common::fs::FilterNotNamedSet>();

		notNamedSet->addObject(L"account-history.json");
		notNamedSet->addObject(L"settings.json");
		notNamedSet->addObject(L"device.json");

		files.addFilter(std::move(notNamedSet));
		files.addFilter(std::make_unique<common::fs::FilterFiles>());

		WIN32_FIND_DATAW file;

		while (files.next(file))
		{
			const auto source = std::filesystem::path(files.getDirectory()).append(file.cFileName);
			const auto target = std::filesystem::path(newCacheDir).append(file.cFileName);
			std::filesystem::rename(source, target);
		}
	}

	//
	// This fails unless the directory is empty. Settings remain in this directory.
	//
	RemoveDirectoryW(std::wstring(L"\\\\?\\").append(oldCacheDir).c_str());
}

void RemoveLogsCacheCurrentUser()
{
	const auto localAppData = common::fs::GetKnownFolderPath(FOLDERID_LocalAppData);
	const auto appdir = std::filesystem::path(localAppData).append(L"Mullvad VPN");

	std::filesystem::remove_all(appdir);

	const auto roamingAppData = common::fs::GetKnownFolderPath(FOLDERID_RoamingAppData);
	const auto roamingAppdir = std::filesystem::path(roamingAppData).append(L"Mullvad VPN");

	std::error_code dummy;
	std::filesystem::remove_all(roamingAppdir, dummy);
}

void RemoveLogsCacheOtherUsers()
{
	//
	// Determine relative path to "local app data" from home directory.
	//
	// Beware, the local app data path may be overriden from its default location
	// as a node somewhere beneath the home directory.
	//

	auto localAppData = common::fs::GetKnownFolderPath(FOLDERID_LocalAppData);
	auto roamingAppData = common::fs::GetKnownFolderPath(FOLDERID_RoamingAppData);
	auto homeDir = common::fs::GetKnownFolderPath(FOLDERID_Profile);

	//
	// Tokenize to get rid of slashes pointing in different directions.
	//
	auto localAppDataTokens = common::string::Tokenize(localAppData, L"\\/");
	auto roamingAppDataTokens = common::string::Tokenize(roamingAppData, L"\\/");
	auto homeDirTokens = common::string::Tokenize(homeDir, L"\\/");

	auto equalTokensCount = EqualTokensCount(
		localAppDataTokens.begin(),
		localAppDataTokens.end(),
		homeDirTokens.begin(),
		homeDirTokens.end()
	);

	//
	// Abort if "local app data" is not beneath home dir.
	//
	if (equalTokensCount < homeDirTokens.size())
	{
		return;
	}

	auto relativeLocalAppData = std::make_pair(std::next(localAppDataTokens.begin(), equalTokensCount), localAppDataTokens.end());

	using StringVectorConstIter = std::vector<std::wstring>::const_iterator;
	std::optional<std::pair<StringVectorConstIter, StringVectorConstIter>> relativeRoamingAppData;

	const auto roamingTokensCount = EqualTokensCount(
		roamingAppDataTokens.begin(),
		roamingAppDataTokens.end(),
		homeDirTokens.begin(),
		homeDirTokens.end()
	);

	if (roamingTokensCount >= homeDirTokens.size())
	{
		relativeRoamingAppData = std::make_optional(std::make_pair(std::next(roamingAppDataTokens.cbegin(), equalTokensCount), roamingAppDataTokens.cend()));
	}

	auto currentUser = *homeDirTokens.rbegin();

	//
	// Find all other users and construct the most plausible path for their
	// respective app data dirs.
	//

	auto parentHomeDir = common::fs::GetKnownFolderPath(FOLDERID_UserProfiles);

	common::fs::FileEnumerator files(parentHomeDir);

	files.addFilter(std::make_unique<common::fs::FilterDirectories>());
	files.addFilter(std::make_unique<common::fs::FilterNotRelativeDirs>());

	auto notNamedSet = std::make_unique<common::fs::FilterNotNamedSet>();

	notNamedSet->addObject(std::wstring(currentUser));
	notNamedSet->addObject(L"All Users"); // Redirects to 'c:\programdata'.
	notNamedSet->addObject(L"Public"); // Shared documents, not an actual user or user template.

	files.addFilter(std::move(notNamedSet));

	WIN32_FIND_DATAW file;

	while (files.next(file))
	{
		const auto userLocalAppData = ConstructUserPath(files.getDirectory(), file.cFileName, relativeLocalAppData);
		const auto target = std::filesystem::path(userLocalAppData).append(L"Mullvad VPN");

		std::error_code dummy;
		std::filesystem::remove_all(target, dummy);

		if (relativeRoamingAppData.has_value())
		{
			const auto userRoamingAppData = ConstructUserPath(files.getDirectory(), file.cFileName, relativeRoamingAppData.value());
			const auto roamingTarget = std::filesystem::path(userRoamingAppData).append(L"Mullvad VPN");

			std::filesystem::remove_all(roamingTarget, dummy);
		}
	}
}

void RemoveLogsServiceUser()
{
	const auto programData = common::fs::GetKnownFolderPath(FOLDERID_ProgramData);
	const auto appdir = std::filesystem::path(programData).append(L"Mullvad VPN");

	{
		common::fs::FileEnumerator files(appdir);
		files.addFilter(std::make_unique<common::fs::FilterFiles>());

		WIN32_FIND_DATAW file;

		while (files.next(file))
		{
			const auto target = std::filesystem::path(files.getDirectory()).append(file.cFileName);

			std::error_code dummy;
			std::filesystem::remove(target, dummy);
		}
	}

	RemoveDirectoryW(std::wstring(L"\\\\?\\").append(appdir).c_str());
}

void RemoveCacheServiceUser()
{
	const auto cacheDir = GetSystemCacheDirectory();

	std::error_code dummy;
	std::filesystem::remove_all(cacheDir, dummy);

	const auto appdir = cacheDir.parent_path();
	RemoveDirectoryW(std::wstring(L"\\\\?\\").append(appdir).c_str());
}

void RemoveSettingsServiceUser()
{
	const auto localAppData = GetSystemUserLocalAppData();
	const auto mullvadAppData = std::filesystem::path(localAppData).append(L"Mullvad VPN");

	common::fs::ScopedNativeFileSystem nativeFileSystem;

	common::security::AddAdminToObjectDacl(mullvadAppData, SE_FILE_OBJECT);

	std::filesystem::remove_all(mullvadAppData);
}

void RemoveRelayCacheServiceUser()
{
	const auto cacheFile = GetSystemCacheDirectory().append(L"relays.json");
	std::filesystem::remove(cacheFile);
}

void RemoveApiAddressCacheServiceUser()
{
	const auto cacheFile = GetSystemCacheDirectory().append(L"api-ip-address.txt");
	std::filesystem::remove(cacheFile);
}

}
