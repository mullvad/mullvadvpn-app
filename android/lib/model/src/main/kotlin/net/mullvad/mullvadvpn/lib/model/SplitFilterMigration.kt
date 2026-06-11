package net.mullvad.mullvadvpn.lib.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

@Parcelize data class SplitFilterMigration(val scenario: Scenario) : Parcelable
