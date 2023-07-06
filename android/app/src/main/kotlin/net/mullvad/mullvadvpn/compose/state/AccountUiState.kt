package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.util.capitalizeFirstCharOfEachWord
import net.mullvad.mullvadvpn.util.toExpiryDateString
import org.joda.time.DateTime

data class AccountUiState(
    private val _deviceName: String,
    val accountNumber: String,
    val accountExpiry: DateTime?
) {
    val deviceName: String
        get() = _deviceName.capitalizeFirstCharOfEachWord()
    val expiryString: String
        get() = accountExpiry?.toExpiryDateString() ?: ""
}
