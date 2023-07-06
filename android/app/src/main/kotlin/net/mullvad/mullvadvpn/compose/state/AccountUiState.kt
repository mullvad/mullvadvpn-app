package net.mullvad.mullvadvpn.compose.state

import java.text.DateFormat
import org.joda.time.DateTime

data class AccountUiState(
    val deviceName: String,
    val accountNumber: String,
    var showAccountNumber: Boolean,
    var accountExpiry: DateTime?
) {
    private val dateStyle = DateFormat.MEDIUM
    private val timeStyle = DateFormat.SHORT
    private val expiryFormatter = DateFormat.getDateTimeInstance(dateStyle, timeStyle)

    var expiryString = accountExpiry?.toDate()?.let { expiryFormatter.format(it) } ?: ""
}
