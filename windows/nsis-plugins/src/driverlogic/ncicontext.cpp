#include "stdafx.h"
#include "ncicontext.h"
#include <stdexcept>

NciContext::NciContext()
{
	dllHandle = LoadLibraryW(L"nci.dll");
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

	if (0 != m_nciGetConnectionName(&guid, nullptr, 0, &nameLen))
	{
		// TODO: = GetLastError()
		throw std::runtime_error("nciGetConnectionName() failed");
	}

	std::wstring buffer;
	buffer.resize(nameLen / sizeof(wchar_t));

	DWORD capacity = static_cast<DWORD>(buffer.capacity() * sizeof(wchar_t));

	if (0 != m_nciGetConnectionName(&guid, &buffer[0], capacity, nullptr))
	{
		// TODO: = GetLastError()
		throw std::runtime_error("nciGetConnectionName() failed");
	}

	return buffer;
}

void NciContext::setConnectionName(const GUID& guid, const wchar_t* newName)
{
	const auto status = m_nciSetConnectionName(&guid, newName);
	if (0 != status)
	{
		// TODO: = GetLastError()
		throw std::runtime_error("nciSetConnectionName() failed");
	}
}
