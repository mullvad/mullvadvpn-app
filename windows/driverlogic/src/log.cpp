#include "stdafx.h"
#include "log.h"
#include <iostream>

void Log(const wchar_t *str)
{
	std::wcout << str << std::endl;
}

void Log(const std::wstring &str)
{
	Log(str.c_str());
}
