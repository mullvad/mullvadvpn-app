package net.mullvad.mullvadvpn.compose.state

import java.text.DateFormat
import net.mullvad.mullvadvpn.util.capitalizeFirstCharOfEachWord
import net.mullvad.mullvadvpn.util.toExpiryDateString
import org.joda.time.DateTime

class AccountUiState(
    deviceName: String,
    val accountNumber: String,
    var accountExpiry: DateTime?
) {
    val deviceName = deviceName.capitalizeFirstCharOfEachWord()
    private val dateStyle = DateFormat.MEDIUM
    private val timeStyle = DateFormat.SHORT
    private val expiryFormatter = DateFormat.getDateTimeInstance(dateStyle, timeStyle)

    var expiryString = accountExpiry?.toExpiryDateString()?: ""
}
