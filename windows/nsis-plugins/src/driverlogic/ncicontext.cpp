#include "stdafx.h"
#include "ncicontext.h"
#include <libcommon/error.h>
#include <libcommon/filesystem.h>
#include <filesystem>
#include <stdexcept>

NciContext::NciContext()
{
	std::wstring systemDir = common::fs::GetKnownFolderPath(
		FOLDERID_System,
		KF_FLAG_DEFAULT,
		nullptr
	);
	const auto lsassPath = std::filesystem::path(systemDir).append(L"nci.dll");

	dllHandle = LoadLibraryW(lsassPath.c_str());

	if (nullptr == dllHandle)
	{
		throw std::runtime_error("Failed to load nci.dll");
	}

	m_nciGetConnectionName = reinterpret_cast<nciGetConnectionNameFunc>(
		GetProcAddress(dllHandle, "NciGetConnectionName"));

	if (nullptr == m_nciGetConnectionName)
	{
		FreeLibrary(dllHandle);
		throw std::runtime_error("Failed to obtain pointer to nciGetConnectionName");
	}

	m_nciSetConnectionName = reinterpret_cast<nciSetConnectionNameFunc>(
		GetProcAddress(dllHandle, "NciSetConnectionName"));

	if (nullptr == m_nciSetConnectionName)
	{
		FreeLibrary(dllHandle);
		throw std::runtime_error("Failed to obtain pointer to nciSetConnectionName");
	}
}

NciContext::~NciContext()
{
	FreeLibrary(dllHandle);
}

std::wstring NciContext::getConnectionName(const GUID& guid)
{
	DWORD nameLen = 0;
	DWORD status = m_nciGetConnectionName(&guid, nullptr, 0, &nameLen);

	if (0 != status)
	{
		common::error::Throw(
			"NciGetConnectionName() failed",
			status
		);
	}

	std::wstring buffer;
	buffer.resize(nameLen / sizeof(wchar_t));

	DWORD capacity = static_cast<DWORD>(buffer.capacity() * sizeof(wchar_t));
	status = m_nciGetConnectionName(&guid, &buffer[0], capacity, nullptr);

	if (0 != status)
	{
		common::error::Throw(
			"NciGetConnectionName() failed",
			status
		);
	}

	return buffer;
}

void NciContext::setConnectionName(const GUID& guid, const wchar_t* newName)
{
	const auto status = m_nciSetConnectionName(&guid, newName);
	if (0 != status)
	{
		common::error::Throw(
			"NciSetConnectionName() failed",
			status
		);
	}
}
