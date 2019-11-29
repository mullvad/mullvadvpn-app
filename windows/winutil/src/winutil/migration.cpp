#include "stdafx.h"
#include "migration.h"
#include <libcommon/filesystem.h>
#include <filesystem>
#include <stdexcept>

namespace migration {

//
// This is being called in a x64 SYSTEM user context
//
MigrationStatus MigrateAfterWindowsUpdate()
{
	const auto localAppData = common::fs::GetKnownFolderPath(FOLDERID_LocalAppData, KF_FLAG_DEFAULT, nullptr);
	const auto mullvadAppData = std::filesystem::path(localAppData).append(L"Mullvad VPN");

	//
	// The main settings file is 'settings.json'
	// If this file is present inside 'mullvadAppData' we should abort the migration
	//

	const auto settingsFile = std::filesystem::path(mullvadAppData).append(L"settings.json");

	if (std::filesystem::exists(settingsFile))
	{
		return MigrationStatus::Aborted;
	}

	//
	// Validate backup location path and ownership
	//

	const auto backupRoot = mullvadAppData.root_path().append(L"windows.old");
	const auto backupMullvadAppData = backupRoot / mullvadAppData.relative_path();

	if (false == std::filesystem::exists(backupMullvadAppData))
	{
		return MigrationStatus::NothingToMigrate;
	}

	DWORD bufferSize = 128;
	std::vector<uint8_t> buffer(bufferSize);

	for (;;)
	{
		if (FALSE == GetFileSecurityW(backupRoot.c_str(), OWNER_SECURITY_INFORMATION,
			&buffer[0], static_cast<DWORD>(buffer.size()), &bufferSize))
		{
			if (ERROR_INSUFFICIENT_BUFFER == GetLastError())
			{
				buffer.resize(bufferSize);
				continue;
			}

			throw std::runtime_error("Could not acquire security descriptor of backup directory");
		}

		break;
	}

	SID *sid = nullptr;
	BOOL ownerDefaulted = FALSE;

	if (FALSE == GetSecurityDescriptorOwner(reinterpret_cast<SECURITY_DESCRIPTOR *>(&buffer[0]),
		reinterpret_cast<PSID *>(&sid), &ownerDefaulted))
	{
		throw std::runtime_error("Could not determine owner of backup directory");
	}

	if (FALSE == IsWellKnownSid(sid, WinLocalSystemSid)
		&& FALSE == IsWellKnownSid(sid, WinBuiltinAdministratorsSid))
	{
		throw std::runtime_error("Backup directory is not owned by SYSTEM or Built-in Administrators");
	}

	//
	// Ensure destination directory exists
	//

	if (false == std::filesystem::exists(mullvadAppData)
		&& false == std::filesystem::create_directory(mullvadAppData))
	{
		throw std::runtime_error("Could not create destination directory during migration");
	}

	//
	// Specify files that need to be copied over
	//

	struct FileMigration
	{
		std::wstring filename;
		bool required;
	};

	const FileMigration filesToMigrate[] = {
		{ L"settings.json", true },
		{ L"account-history.json", false },
	};

	//
	// Copy and delete files
	//

	bool copyStatus = true;

	for (const auto file : filesToMigrate)
	{
		const auto from = std::filesystem::path(backupMullvadAppData).append(file.filename);
		const auto to = std::filesystem::path(mullvadAppData).append(file.filename);

		std::error_code error;

		if (std::filesystem::copy_file(from, to, std::filesystem::copy_options::overwrite_existing | std::filesystem::copy_options::skip_symlinks, error))
		{
			std::filesystem::remove(from, error);
		}
		else if (file.required)
		{
			copyStatus = false;
		}
	}

	if (false == copyStatus)
	{
		throw std::runtime_error("Failed to copy files during migration");
	}

	return MigrationStatus::Success;
}

}
