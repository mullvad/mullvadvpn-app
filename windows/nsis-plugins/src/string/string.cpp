#include "stdafx.h"
#include <nsis/pluginapi.h>
#include <libcommon/error.h>
#include <string>

// Suppress warnings caused by broken legacy code
#pragma warning (push)
#pragma warning (disable: 4005)
#include <nsis/pluginapi.h>
#pragma warning (pop)

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

} // anonymous namespace

//
// Find "string" "substring" begin_offset
//
// Return the position of "substring" in "string", starting from 'begin_offset'.
// If the substring is not found or if an error occurs, return -1.
//

void __declspec(dllexport) NSISCALL Find
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
		const auto searchString = PopString();
		const auto substring = PopString();
		const auto offset = popint();
		const auto position = searchString.find(substring, offset);

		if (std::wstring::npos == position)
		{
			pushint(-1);
			return;
		}

		pushint((int)position);
	}
	catch (const std::exception &)
	{
		pushint(-1);
	}
	catch (...)
	{
		pushint(-1);
	}
}
