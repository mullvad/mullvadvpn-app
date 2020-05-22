package net.mullvad.mullvadvpn.service

import kotlin.math.min
import kotlinx.coroutines.delay
import net.mullvad.mullvadvpn.model.GetAccountDataResult
import net.mullvad.mullvadvpn.util.JobTracker
import net.mullvad.talpid.util.EventNotifier
import org.joda.time.DateTime
import org.joda.time.format.DateTimeFormat

class AccountCache(val daemon: MullvadDaemon, val settingsListener: SettingsListener) {
    companion object {
        public val EXPIRY_FORMAT = DateTimeFormat.forPattern("YYYY-MM-dd HH:mm:ss z")
    }

    private val jobTracker = JobTracker()

    private var accountNumber: String? = null
        set(value) {
            field = value
            onAccountNumberChange.notify(value)
        }

    private var accountExpiry: DateTime? = null
        set(value) {
            field = value
            onAccountExpiryChange.notify(value)
        }

    val onAccountNumberChange = EventNotifier<String?>(null)
    val onAccountExpiryChange = EventNotifier<DateTime?>(null)

    init {
        settingsListener.accountNumberNotifier.subscribe(this) { accountNumber ->
            handleNewAccountNumber(accountNumber)
        }
    }

    fun fetchAccountExpiry() {
        accountNumber?.let { account ->
            jobTracker.newBackgroundJob("fetch") {
                var retryAttempt = 0

                do {
                    val result = daemon.getAccountData(account)

                    if (result is GetAccountDataResult.Ok) {
                        handleNewExpiry(account, result.accountData.expiry)
                        break
                    } else if (result is GetAccountDataResult.InvalidAccount) {
                        break
                    }

                    retryAttempt += 1
                    delay(calculateRetryFetchDelay(retryAttempt))
                } while (onAccountExpiryChange.hasListeners())
            }
        }
    }

    fun onDestroy() {
        settingsListener.accountNumberNotifier.unsubscribe(this)
        jobTracker.cancelAllJobs()
    }

    private fun handleNewAccountNumber(newAccountNumber: String?) {
        synchronized(this) {
            accountNumber = newAccountNumber
            accountExpiry = null

            fetchAccountExpiry()
        }
    }

    private fun handleNewExpiry(accountNumberUsedForFetch: String, expiryString: String) {
        synchronized(this) {
            if (accountNumber === accountNumberUsedForFetch) {
                accountExpiry = DateTime.parse(expiryString, EXPIRY_FORMAT)
            }
        }
    }

    private fun calculateRetryFetchDelay(retryAttempt: Int): Long {
        // delay in seconds = 2 ^ retryAttempt capped at 2^13 (8192)
        val exponent = min(retryAttempt, 13)

        return (1L shl exponent) * 1000L
    }
}
