package net.mullvad.mullvadvpn.dataproxy

import kotlinx.coroutines.Deferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.MullvadDaemon
import org.joda.time.DateTime
import org.joda.time.format.DateTimeFormat

val EXPIRY_FORMAT = DateTimeFormat.forPattern("YYYY-MM-dd HH:mm:ss z")

class AccountCache(val settingsListener: SettingsListener, val daemon: Deferred<MullvadDaemon>) {
    private var fetchJob: Job? = null
    private var accountNumber: String? = null
    private var accountExpiry: DateTime? = null

    var onAccountDataChange: ((String?, DateTime?) -> Unit)? = null
        set(value) {
            synchronized(this) {
                field = value
                notifyChange()
            }
        }

    init {
        settingsListener.onAccountNumberChange = { accountNumber ->
            handleNewAccountNumber(accountNumber)
        }
    }

    fun refetch() {
        fetchJob?.cancel()
        fetchJob = fetchAccountExpiry()
    }

    fun onDestroy() {
        settingsListener.onAccountNumberChange = null

        fetchJob?.cancel()
    }

    private fun handleNewAccountNumber(newAccountNumber: String?) {
        synchronized(this) {
            accountNumber = newAccountNumber
            accountExpiry = null

            notifyChange()
            refetch()
        }
    }

    private fun fetchAccountExpiry() = GlobalScope.launch(Dispatchers.Default) {
        val accountNumber = this@AccountCache.accountNumber
        val accountData = accountNumber?.let { account ->
            daemon.await().getAccountData(account)
        }

        synchronized(this@AccountCache) {
            if (this@AccountCache.accountNumber === accountNumber) {
                accountExpiry = accountData?.expiry?.let { expiry ->
                    DateTime.parse(expiry, EXPIRY_FORMAT)
                }

                notifyChange()
            }
        }
    }

    private fun notifyChange() {
        onAccountDataChange?.invoke(accountNumber, accountExpiry)
    }
}
