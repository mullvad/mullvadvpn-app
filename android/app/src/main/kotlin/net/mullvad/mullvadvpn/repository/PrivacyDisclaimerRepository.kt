package net.mullvad.mullvadvpn.repository

import android.content.SharedPreferences

private const val IS_PRIVACY_DISCLOSURE_ACCEPTED_KEY = "is_privacy_disclosure_accepted"

class PrivacyDisclaimerRepository(private val sharedPreferences: SharedPreferences) {
    fun hasAcceptedPrivacyDisclosure(): Boolean {
        return sharedPreferences.getBoolean(IS_PRIVACY_DISCLOSURE_ACCEPTED_KEY, false)
    }

    fun setPrivacyDisclosureAccepted() {
        sharedPreferences.edit().putBoolean(IS_PRIVACY_DISCLOSURE_ACCEPTED_KEY, true).apply()
    }
}
