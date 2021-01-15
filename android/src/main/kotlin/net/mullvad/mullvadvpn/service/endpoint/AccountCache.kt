package net.mullvad.mullvadvpn.service.endpoint

import kotlinx.coroutines.delay
import net.mullvad.mullvadvpn.model.GetAccountDataResult
import net.mullvad.mullvadvpn.model.LoginStatus
import net.mullvad.mullvadvpn.util.ExponentialBackoff
import net.mullvad.mullvadvpn.util.JobTracker
import net.mullvad.talpid.util.EventNotifier
import org.joda.time.DateTime
import org.joda.time.format.DateTimeFormat

class AccountCache(private val endpoint: ServiceEndpoint) {
    companion object {
        public val EXPIRY_FORMAT = DateTimeFormat.forPattern("YYYY-MM-dd HH:mm:ss z")

        // Number of retry attempts to check for a changed expiry before giving up.
        // Current value will force the cache to keep fetching for about four minutes or until a new
        // expiry value is received.
        // This is only used if the expiry was invalidated and fetching a new expiry returns the
        // same value as before the invalidation.
        private const val MAX_INVALIDATED_RETRIES = 7
    }

    private val daemon
        get() = endpoint.intermittentDaemon

    val onAccountNumberChange = EventNotifier<String?>(null)
    val onAccountExpiryChange = EventNotifier<DateTime?>(null)
    val onAccountHistoryChange = EventNotifier<List<String>>(listOf<String>())
    val onLoginStatusChange = EventNotifier<LoginStatus?>(null)

    var newlyCreatedAccount = false
        private set

    private val jobTracker = JobTracker()

    private var accountNumber by onAccountNumberChange.notifiable()
    private var accountExpiry by onAccountExpiryChange.notifiable()
    private var accountHistory by onAccountHistoryChange.notifiable()

    private var createdAccountExpiry: DateTime? = null
    private var oldAccountExpiry: DateTime? = null

    var loginStatus by onLoginStatusChange.notifiable()
        private set

    init {
        endpoint.settingsListener.accountNumberNotifier.subscribe(this) { accountNumber ->
            handleNewAccountNumber(accountNumber)
        }
    }

    suspend fun createNewAccount() {
        newlyCreatedAccount = true
        createdAccountExpiry = null

        daemon.await().createNewAccount()
    }

    suspend fun login(account: String) {
        if (account != accountNumber) {
            markAccountAsNotNew()
            daemon.await().setAccount(account)
        }
    }

    fun fetchAccountExpiry() {
        synchronized(this) {
            accountNumber?.let { account ->
                jobTracker.newBackgroundJob("fetch") {
                    val delays = ExponentialBackoff().apply {
                        cap = 2 /* h */ * 60 /* min */ * 60 /* s */ * 1000 /* ms */
                    }

                    do {
                        val result = daemon.await().getAccountData(account)

                        if (result is GetAccountDataResult.Ok) {
                            val expiry = result.accountData.expiry
                            val retryAttempt = delays.iteration

                            if (handleNewExpiry(account, expiry, retryAttempt)) {
                                break
                            }
                        } else if (result is GetAccountDataResult.InvalidAccount) {
                            break
                        }

                        delay(delays.next())
                    } while (onAccountExpiryChange.hasListeners())
                }
            }
        }
    }

    fun invalidateAccountExpiry(accountExpiryToInvalidate: DateTime) {
        synchronized(this) {
            if (accountExpiry == accountExpiryToInvalidate) {
                oldAccountExpiry = accountExpiryToInvalidate
                fetchAccountExpiry()
            }
        }
    }

    fun removeAccountFromHistory(accountToken: String) {
        jobTracker.newBackgroundJob("removeAccountFromHistory $accountToken") {
            daemon.await().removeAccountFromHistory(accountToken)
            fetchAccountHistory()
        }
    }

    fun onDestroy() {
        endpoint.settingsListener.accountNumberNotifier.unsubscribe(this)
        jobTracker.cancelAllJobs()

        onAccountNumberChange.unsubscribeAll()
        onAccountExpiryChange.unsubscribeAll()
        onAccountHistoryChange.unsubscribeAll()
        onLoginStatusChange.unsubscribeAll()
    }

    private fun fetchAccountHistory() {
        jobTracker.newBackgroundJob("fetchHistory") {
            daemon.await().getAccountHistory()?.let { history ->
                accountHistory = history
            }
        }
    }

    private fun markAccountAsNotNew() {
        newlyCreatedAccount = false
        createdAccountExpiry = null
    }

    private fun handleNewAccountNumber(newAccountNumber: String?) {
        synchronized(this) {
            accountExpiry = null
            accountNumber = newAccountNumber

            loginStatus = newAccountNumber?.let { account ->
                LoginStatus(account, null, newlyCreatedAccount)
            }

            fetchAccountExpiry()
            fetchAccountHistory()
        }
    }

    private fun handleNewExpiry(
        accountNumberUsedForFetch: String,
        expiryString: String,
        retryAttempt: Int
    ): Boolean {
        synchronized(this) {
            if (accountNumber !== accountNumberUsedForFetch) {
                return true
            }

            val newAccountExpiry = DateTime.parse(expiryString, EXPIRY_FORMAT)

            if (newAccountExpiry != oldAccountExpiry || retryAttempt >= MAX_INVALIDATED_RETRIES) {
                accountExpiry = newAccountExpiry
                oldAccountExpiry = null

                loginStatus = loginStatus?.let { currentStatus ->
                    LoginStatus(currentStatus.account, newAccountExpiry, currentStatus.isNewAccount)
                }

                if (accountExpiry != null && newlyCreatedAccount) {
                    if (createdAccountExpiry == null) {
                        createdAccountExpiry = accountExpiry
                    } else if (accountExpiry != createdAccountExpiry) {
                        markAccountAsNotNew()
                    }
                }

                return true
            }

            return false
        }
    }
}
