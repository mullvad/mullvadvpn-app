package net.mullvad.mullvadvpn.service.endpoint

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.channels.ClosedReceiveChannelException
import kotlinx.coroutines.channels.actor
import kotlinx.coroutines.channels.trySendBlocking
import kotlinx.coroutines.flow.collect
import net.mullvad.mullvadvpn.ipc.Event
import net.mullvad.mullvadvpn.ipc.Request
import net.mullvad.mullvadvpn.model.AccountCreationResult
import net.mullvad.mullvadvpn.model.AccountExpiry
import net.mullvad.mullvadvpn.model.AccountHistory
import net.mullvad.mullvadvpn.model.GetAccountDataResult
import net.mullvad.mullvadvpn.util.JobTracker
import net.mullvad.mullvadvpn.util.parseAsDateTime
import net.mullvad.talpid.util.EventNotifier

class AccountCache(private val endpoint: ServiceEndpoint) {
    companion object {
        private sealed class Command {
            object CreateAccount : Command()
            data class Login(val account: String) : Command()
            object Logout : Command()
        }
    }

    private val commandChannel = spawnActor()

    private val daemon
        get() = endpoint.intermittentDaemon

    val onAccountExpiryChange = EventNotifier<AccountExpiry>(AccountExpiry.Missing)
    val onAccountHistoryChange = EventNotifier<AccountHistory>(AccountHistory.Missing)

    private val jobTracker = JobTracker()

    private var accountExpiry by onAccountExpiryChange.notifiable()
    private var accountHistory by onAccountHistoryChange.notifiable()

    private var cachedAccountToken: String? = null
    private var cachedCreatedAccountToken: String? = null

    val isNewAccount: Boolean
        get() = cachedAccountToken == cachedCreatedAccountToken

    init {
        jobTracker.newBackgroundJob("autoFetchAccountExpiry") {
            daemon.await().deviceStateUpdates.collect { deviceState ->
                accountExpiry = deviceState.token()
                    .also { cachedAccountToken = it }
                    ?.let { fetchAccountExpiry(it) } ?: AccountExpiry.Missing
            }
        }

        onAccountHistoryChange.subscribe(this) { history ->
            endpoint.sendEvent(Event.AccountHistoryEvent(history))
        }

        onAccountExpiryChange.subscribe(this) {
            endpoint.sendEvent(Event.AccountExpiryEvent(it))
        }

        endpoint.dispatcher.apply {
            registerHandler(Request.CreateAccount::class) { _ ->
                commandChannel.trySendBlocking(Command.CreateAccount)
            }

            registerHandler(Request.Login::class) { request ->
                request.account?.let { account ->
                    commandChannel.trySendBlocking(Command.Login(account))
                }
            }

            registerHandler(Request.Logout::class) { _ ->
                commandChannel.trySendBlocking(Command.Logout)
            }

            registerHandler(Request.FetchAccountExpiry::class) { _ ->
                jobTracker.newBackgroundJob("fetchAccountExpiry") {
                    accountExpiry = cachedAccountToken
                        ?.let { fetchAccountExpiry(it) } ?: AccountExpiry.Missing
                }
            }

            registerHandler(Request.FetchAccountHistory::class) { _ ->
                jobTracker.newBackgroundJob("fetchAccountHistory") {
                    accountHistory = fetchAccountHistory()
                }
            }

            registerHandler(Request.ClearAccountHistory::class) { _ ->
                jobTracker.newBackgroundJob("clearAccountHistory") {
                    clearAccountHistory()
                }
            }
        }
    }

    fun onDestroy() {
        jobTracker.cancelAllJobs()

        onAccountExpiryChange.unsubscribeAll()
        onAccountHistoryChange.unsubscribeAll()

        commandChannel.close()
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

    private suspend fun clearAccountHistory() {
        daemon.await().clearAccountHistory()
        accountHistory = fetchAccountHistory()
    }

    private suspend fun doCreateAccount() {
        daemon.await().createNewAccount()
            .also { newAccountToken ->
                cachedCreatedAccountToken = newAccountToken
            }
            .let { newAccountToken ->
                if (newAccountToken != null) {
                    AccountCreationResult.Success(newAccountToken)
                } else {
                    AccountCreationResult.Failure
                }
            }
            .also { result ->
                endpoint.sendEvent(Event.AccountCreationEvent(result))
            }
    }

    private suspend fun doLogin(account: String) {
        daemon.await().loginAccount(account).also { result ->
            endpoint.sendEvent(Event.LoginEvent(result))
        }
    }

    private suspend fun doLogout() {
        daemon.await().logoutAccount()
        accountHistory = fetchAccountHistory()
    }

    private suspend fun fetchAccountHistory(): AccountHistory {
        return daemon.await().getAccountHistory().let { history ->
            if (history != null) {
                AccountHistory.Available(history)
            } else {
                AccountHistory.Missing
            }
        }
    }

    private suspend fun fetchAccountExpiry(accountToken: String): AccountExpiry {
        return fetchAccountData(accountToken).let { result ->
            if (result is GetAccountDataResult.Ok) {
                result.accountData.expiry.parseAsDateTime()?.let { parsedDateTime ->
                    AccountExpiry.Available(parsedDateTime)
                } ?: AccountExpiry.Missing
            } else {
                AccountExpiry.Missing
            }
        }
    }

    private suspend fun fetchAccountData(accountToken: String): GetAccountDataResult {
        return daemon.await().getAccountData(accountToken)
    }
}
