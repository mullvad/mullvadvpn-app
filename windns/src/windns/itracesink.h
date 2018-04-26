#pragma once

#include <sstream>

struct ITraceSink
{
	virtual ~ITraceSink() = 0
	{
	}

	virtual void trace(const wchar_t *msg) = 0;
};

struct NullTraceSink : public ITraceSink
{
	void trace(const wchar_t *) override
	{
	}
};

#ifdef _DEBUG
#define TRACING_ENABLED 1
#else
#define TRACING_ENABLED 0
#endif

#if TRACING_ENABLED == 1

#include "macroargument.h"
#define XTRACE(...) VFUNC(XTRACE, __VA_ARGS__)

#define XTRACE1(x)\
{\
std::wstringstream xtrace_ss;\
xtrace_ss << __FUNCTIONW__ << L": " << x;\
m_traceSink->trace(xtrace_ss.str().c_str());\
}

#define XTRACE2(x, y)\
{\
std::wstringstream xtrace_ss;\
xtrace_ss << __FUNCTIONW__ << L": " << x << L" " << y;\
m_traceSink->trace(xtrace_ss.str().c_str());\
}

#else
#define XTRACE(...)
#endif
