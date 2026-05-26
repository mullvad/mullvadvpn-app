package net.mullvad.mullvadvpn.lib.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

@Parcelize
enum class MultihopMigrationState : Parcelable {
    ON_TO_ALWAYS,
    OFF_TO_NEVER,
    OFF_TO_WHEN_NEEDED,
    OFF_TO_ALWAYS
}
