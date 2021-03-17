#pragma once

#include <string>
#include "device.h"

enum class DRIVER_UPGRADE_STATUS
{
	WOULD_DOWNGRADE,
	WOULD_INSTALL_SAME_VERSION,
	WOULD_UPGRADE
};

DRIVER_UPGRADE_STATUS
EvaluateDriverUpgrade
(
	const std::wstring &existingVersion,
	const std::wstring &proposedVersion
);

std::wstring
InfGetDriverVersion
(
	const std::wstring &filePath
);

std::wstring
GetDriverVersion
(
	const EnumeratedDevice &device
);
