#pragma once

namespace update
{

// Checks whether SHA-2 signatures can be verified correctly
// on Windows 7. Without this patch, the driver cannot be
// installed from a service.
bool HasSetupApiSha2Fix();

}
