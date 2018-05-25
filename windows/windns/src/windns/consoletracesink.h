#pragma once

#include "itracesink.h"

struct ConsoleTraceSink : public ITraceSink
{
	void trace(const wchar_t *msg) override;
};
