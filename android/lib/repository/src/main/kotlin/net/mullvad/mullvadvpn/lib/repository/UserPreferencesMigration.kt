package net.mullvad.mullvadvpn.lib.repository

import android.content.Context
import androidx.datastore.core.DataMigration
import androidx.datastore.migrations.SharedPreferencesMigration
import androidx.datastore.migrations.SharedPreferencesView
import net.mullvad.mullvadvpn.repository.UserPreferences

private const val IS_PRIVACY_DISCLOSURE_ACCEPTED_KEY_SHARED_PREF_KEY =
    "is_privacy_disclosure_accepted"

data object UserPreferencesMigration {
    fun migrations(
        context: Context,
        sharedPreferencesName: String,
    ): List<DataMigration<UserPreferences>> =
        listOf(
            SharedPreferencesMigration(
                context,
                sharedPreferencesName = sharedPreferencesName,
                keysToMigrate = setOf(IS_PRIVACY_DISCLOSURE_ACCEPTED_KEY_SHARED_PREF_KEY),
            ) { sharedPrefs: SharedPreferencesView, currentData: UserPreferences ->
                val privacyDisclosureAccepted =
                    sharedPrefs.getBoolean(
                        IS_PRIVACY_DISCLOSURE_ACCEPTED_KEY_SHARED_PREF_KEY,
                        false,
                    )
                currentData
                    .toBuilder()
                    .setIsPrivacyDisclosureAccepted(privacyDisclosureAccepted)
                    .build()
            }
        )
}
