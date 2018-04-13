#pragma once

#include "libcommon/error.h"
#include <winerror.h>

#define VALIDATE_COM(status, operation)\
if(FAILED(status))\
{\
	::common::error::Throw(operation, status);\
}
