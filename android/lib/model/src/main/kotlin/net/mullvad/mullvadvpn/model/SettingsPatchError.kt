package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

@Parcelize
sealed class SettingsPatchError : Parcelable {
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
