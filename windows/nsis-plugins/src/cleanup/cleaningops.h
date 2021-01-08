#pragma once

namespace cleaningops
{

void MigrateCacheServiceUser();

void RemoveLogsCacheCurrentUser();
void RemoveLogsCacheOtherUsers();
void RemoveLogsServiceUser();
void RemoveCacheServiceUser();
void RemoveSettingsServiceUser();

// Remove only the relay cache, leaving other cache files untouched.
// This is useful when updating the app.
void RemoveRelayCacheServiceUser();
void RemoveApiAddressCacheServiceUser();

}
