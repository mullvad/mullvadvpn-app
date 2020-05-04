package net.mullvad.mullvadvpn.dataproxy

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.model.GetAccountDataResult
import net.mullvad.mullvadvpn.service.MullvadDaemon
import net.mullvad.mullvadvpn.service.SettingsListener
import org.joda.time.DateTime
import org.joda.time.format.DateTimeFormat

val EXPIRY_FORMAT = DateTimeFormat.forPattern("YYYY-MM-dd HH:mm:ss z")

class AccountCache(val daemon: MullvadDaemon, val settingsListener: SettingsListener) {
    private val subscriptionId = settingsListener.accountNumberNotifier.subscribe { accountNumber ->
        handleNewAccountNumber(accountNumber)
    }

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

    fun refetch() {
        fetchJob?.cancel()
        fetchJob = fetchAccountExpiry()
    }

    fun onDestroy() {
        settingsListener.accountNumberNotifier.unsubscribe(subscriptionId)

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
            val result = daemon.getAccountData(account)

            when (result) {
                is GetAccountDataResult.Ok -> result.accountData
                else -> null
            }
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
