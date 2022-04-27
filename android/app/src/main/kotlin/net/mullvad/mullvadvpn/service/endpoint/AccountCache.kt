package net.mullvad.mullvadvpn.service.endpoint

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.channels.ClosedReceiveChannelException
import kotlinx.coroutines.channels.actor
import kotlinx.coroutines.channels.sendBlocking
import kotlinx.coroutines.delay
import net.mullvad.mullvadvpn.ipc.Event
import net.mullvad.mullvadvpn.ipc.Request
import net.mullvad.mullvadvpn.model.AccountCreationResult
import net.mullvad.mullvadvpn.model.GetAccountDataResult
import net.mullvad.mullvadvpn.model.LoginResult
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

        private sealed class Command {
            object CreateAccount : Command()
            data class Login(val account: String) : Command()
            object Logout : Command()
        }
    }

    private val commandChannel = spawnActor()

    private val daemon
        get() = endpoint.intermittentDaemon

    val onAccountNumberChange = EventNotifier<String?>(null)
    val onAccountExpiryChange = EventNotifier<DateTime?>(null)
    val onAccountHistoryChange = EventNotifier<String?>(null)
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

        onAccountHistoryChange.subscribe(this) { history ->
            endpoint.sendEvent(Event.AccountHistory(history))
        }

        onLoginStatusChange.subscribe(this) { status ->
            endpoint.sendEvent(Event.LoginStatus(status))
        }

        endpoint.dispatcher.apply {
            registerHandler(Request.CreateAccount::class) { _ ->
                commandChannel.sendBlocking(Command.CreateAccount)
            }

            registerHandler(Request.Login::class) { request ->
                request.account?.let { account ->
                    commandChannel.sendBlocking(Command.Login(account))
                }
            }

            registerHandler(Request.Logout::class) { _ ->
                commandChannel.sendBlocking(Command.Logout)
            }

            registerHandler(Request.FetchAccountExpiry::class) { _ ->
                fetchAccountExpiry()
            }

            registerHandler(Request.InvalidateAccountExpiry::class) { request ->
                invalidateAccountExpiry(request.expiry)
            }

            registerHandler(Request.ClearAccountHistory::class) { _ ->
                clearAccountHistory()
            }
        }
    }

    fun onDestroy() {
        endpoint.settingsListener.accountNumberNotifier.unsubscribe(this)
        jobTracker.cancelAllJobs()

        onAccountNumberChange.unsubscribeAll()
        onAccountExpiryChange.unsubscribeAll()
        onAccountHistoryChange.unsubscribeAll()
        onLoginStatusChange.unsubscribeAll()

        commandChannel.close()
    }

    private fun fetchAccountExpiry() {
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

    private fun invalidateAccountExpiry(accountExpiryToInvalidate: DateTime) {
        synchronized(this) {
            if (accountExpiry == accountExpiryToInvalidate) {
                oldAccountExpiry = accountExpiryToInvalidate
                fetchAccountExpiry()
            }
        }
    }

    private fun clearAccountHistory() {
        jobTracker.newBackgroundJob("clearAccountHistory") {
            daemon.await().clearAccountHistory()
            fetchAccountHistory()
        }
    }

    private fun spawnActor() = GlobalScope.actor<Command>(Dispatchers.Default, Channel.UNLIMITED) {
        try {
            for (command in channel) {
                when (command) {
                    is Command.CreateAccount -> doCreateAccount()
                    is Command.Login -> doLogin(command.account)
                    is Command.Logout -> doLogout()
                }
            }
        } catch (exception: ClosedReceiveChannelException) {
            // Command channel was closed, stop the actor
        }
    }

    private suspend fun doCreateAccount() {
        newlyCreatedAccount = true
        createdAccountExpiry = null

        daemon.await().createNewAccount()
            .let { newAccountNumber ->
                if (newAccountNumber != null) {
                    AccountCreationResult.Success(newAccountNumber)
                } else {
                    AccountCreationResult.Failure
                }
            }
            .also { result ->
                endpoint.sendEvent(Event.AccountCreationEvent(result))
            }
    }

    private suspend fun doLogin(account: String) {
        val loginResult = daemon.await().loginAccount(account)

        val accountExpiryDate = loginResult
            .takeIf { it == LoginResult.Ok }
            .let { daemon.await().getAccountData(account) as? GetAccountDataResult.Ok }
            ?.let { DateTime.parse(it.accountData.expiry, EXPIRY_FORMAT) }

        synchronized(this) {
            markAccountAsNotNew()
            accountNumber = account
            accountExpiry = accountExpiryDate

            loginStatus = LoginStatus(
                account = account,
                expiry = accountExpiryDate,
                isNewAccount = newlyCreatedAccount,
                loginResult
            )
        }
    }

    private suspend fun doLogout() {
        daemon.await().logoutAccount()
        loginStatus = null
        fetchAccountHistory()
    }

    private fun fetchAccountHistory() {
        jobTracker.newBackgroundJob("fetchHistory") {
            daemon.await().getAccountHistory().let { history ->
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
                LoginStatus(account, null, newlyCreatedAccount, null)
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
                    LoginStatus(
                        currentStatus.account,
                        newAccountExpiry,
                        currentStatus.isNewAccount,
                        null
                    )
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
