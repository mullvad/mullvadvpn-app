package net.mullvad.mullvadvpn.lib.model

sealed class SettingsPatchError {
    // E.g hostname is number instead of String
    data class InvalidOrMissingValue(val value: String) : SettingsPatchError()

    // E.g. Unexpected top-level key?
    data class UnknownOrProhibitedKey(val value: String) : SettingsPatchError()

    // Bad JSON
    data object ParsePatch : SettingsPatchError()

    data object RecursionLimit : SettingsPatchError()

    // Patch was deserialized but was not valid domain data?
    data object DeserializePatched : SettingsPatchError()

    // Failed to apply patch
    data object ApplyPatch : SettingsPatchError()
}
