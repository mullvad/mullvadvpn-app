#include "stdafx.h"
#include "consoletracesink.h"
#include <iostream>

void ConsoleTraceSink::trace(const wchar_t *msg)
{
	std::wcout << msg << std::endl;
}
