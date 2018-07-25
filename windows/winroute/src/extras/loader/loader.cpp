#include "stdafx.h"
#include "../../winroute/winroute.h"

int main()
{
	const auto status = GetTapInterfaceIpv6Status(nullptr, nullptr);

    return 0;
}

