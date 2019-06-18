package net.mullvad.mullvadvpn.dataproxy

import kotlinx.coroutines.async
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope

import org.joda.time.format.DateTimeFormat
import org.joda.time.DateTime

import net.mullvad.mullvadvpn.MainActivity

val EXPIRY_FORMAT = DateTimeFormat.forPattern("YYYY-MM-dd HH:mm:ss z")

class AccountCache(val parentActivity: MainActivity) {
    private var daemon = parentActivity.asyncDaemon

    var settings = parentActivity.asyncSettings
        set(value) {
            field = value
            accountNumber = fetchAccountNumber()
            accountExpiry = fetchAccountExpiry()
        }

    var accountNumber = fetchAccountNumber()
        private set
    var accountExpiry = fetchAccountExpiry()
        private set

    fun onDestroy() {
        accountExpiry.cancel()
        accountNumber.cancel()
    }

    private fun fetchAccountNumber() = GlobalScope.async(Dispatchers.Default) {
        settings.await().accountToken
    }

    private fun fetchAccountExpiry() = GlobalScope.async(Dispatchers.Default) {
        val accountNumber = accountNumber.await()

        if (accountNumber != null) {
            val accountData = daemon.await().getAccountData(accountNumber)
            val accountExpiry = accountData?.expiry

            if (accountExpiry != null) {
                DateTime.parse(accountExpiry, EXPIRY_FORMAT)
            } else {
                null
            }
        } else {
            null
        }
    }
}
