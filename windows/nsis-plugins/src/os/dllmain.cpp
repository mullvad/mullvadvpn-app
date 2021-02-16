#include "stdafx.h"
#include <windows.h>

BOOL APIENTRY DllMain(HMODULE, DWORD, LPVOID)
{
	//
	// Avoid doing work in DllMain since the loader lock is held
	//

	return TRUE;
}
