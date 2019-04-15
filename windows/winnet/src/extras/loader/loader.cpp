#include "stdafx.h"
#include "../../winnet/winnet.h"

int main()
{
	const auto status = WinNet_GetTapInterfaceIpv6Status(nullptr, nullptr);

    return 0;
}

