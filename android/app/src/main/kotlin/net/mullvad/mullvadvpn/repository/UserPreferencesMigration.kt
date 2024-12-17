package net.mullvad.mullvadvpn.repository

import android.content.Context
import androidx.datastore.core.DataMigration
import androidx.datastore.migrations.SharedPreferencesMigration
import androidx.datastore.migrations.SharedPreferencesView
import net.mullvad.mullvadvpn.di.APP_PREFERENCES_NAME

private const val IS_PRIVACY_DISCLOSURE_ACCEPTED_KEY_SHARED_PREF_KEY =
    "is_privacy_disclosure_accepted"

data object UserPreferencesMigration {
    fun migrations(context: Context): List<DataMigration<UserPreferences>> =
        listOf(
            SharedPreferencesMigration(context, sharedPreferencesName = APP_PREFERENCES_NAME) {
                sharedPrefs: SharedPreferencesView,
                currentData: UserPreferences ->
                if (
                    sharedPrefs.getBoolean(
                        IS_PRIVACY_DISCLOSURE_ACCEPTED_KEY_SHARED_PREF_KEY,
                        false,
                    )
                ) {
                    currentData.toBuilder().setIsPrivacyDisclosureAccepted(true).build()
                } else {
                    currentData
                }
            }
        )
}
