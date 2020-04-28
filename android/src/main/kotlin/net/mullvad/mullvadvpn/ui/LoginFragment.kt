package net.mullvad.mullvadvpn.ui

import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.TextView
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.model.GetAccountDataResult
import net.mullvad.mullvadvpn.ui.widget.Button
import net.mullvad.mullvadvpn.util.JobTracker

class LoginFragment : ServiceDependentFragment(OnNoService.GoToLaunchScreen) {
    private lateinit var title: TextView
    private lateinit var subtitle: TextView
    private lateinit var loggingInStatus: View
    private lateinit var loggedInStatus: View
    private lateinit var loginFailStatus: View
    private lateinit var accountInput: AccountInput

    private val jobTracker = JobTracker()
    private val loggedIn = CompletableDeferred<Unit>()

    override fun onSafelyCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View {
        val view = inflater.inflate(R.layout.login, container, false)

        view.findViewById<View>(R.id.settings).setOnClickListener { parentActivity.openSettings() }

        title = view.findViewById(R.id.title)
        subtitle = view.findViewById(R.id.subtitle)
        loggingInStatus = view.findViewById(R.id.logging_in_status)
        loggedInStatus = view.findViewById(R.id.logged_in_status)
        loginFailStatus = view.findViewById(R.id.login_fail_status)

        accountInput = AccountInput(view, parentActivity.resources)
        accountInput.onLogin = { accountToken -> login(accountToken) }

        view.findViewById<Button>(R.id.create_account)
            .setOnClickAction("createAccount", jobTracker) { createAccount() }

        fetchHistory()

        return view
    }

    override fun onSafelyResume() {
        jobTracker.newUiJob("advanceToNextScreen") {
            loggedIn.join()
            openConnectScreen()
        }

        fetchHistory()
    }

    override fun onSafelyPause() {
        jobTracker.cancelJob("advanceToNextScreen")
    }

    override fun onSafelyDestroyView() {
        jobTracker.cancelAllJobs()
    }

    private suspend fun createAccount() {
        title.setText(R.string.logging_in_title)
        subtitle.setText(R.string.creating_new_account)

        loggingInStatus.visibility = View.VISIBLE
        loginFailStatus.visibility = View.GONE
        loggedInStatus.visibility = View.GONE

        accountInput.state = LoginState.InProgress

        val accountToken = jobTracker.runOnBackground {
            daemon.createNewAccount()
        }

        if (accountToken == null) {
            loginFailure(R.string.failed_to_create_account)
        } else {
            loggedIn(resources.getString(R.string.account_created))
        }
    }

    private fun login(accountToken: String) {
        title.setText(R.string.logging_in_title)
        subtitle.setText(R.string.logging_in_description)

        loggingInStatus.visibility = View.VISIBLE
        loginFailStatus.visibility = View.GONE
        loggedInStatus.visibility = View.GONE

        accountInput.state = LoginState.InProgress

        performLogin(accountToken)
    }

    private fun fetchHistory() {
        jobTracker.newUiJob("fetchHistory") {
            accountInput.accountHistory = jobTracker.runOnBackground() {
                daemon.getAccountHistory()
            }
        }
    }

    private fun performLogin(accountToken: String) = GlobalScope.launch(Dispatchers.Main) {
        jobTracker.newUiJob("login") {
            val loginSucceeded = jobTracker.runOnBackground {
                val accountDataResult = daemon.getAccountData(accountToken)

                when (accountDataResult) {
                    is GetAccountDataResult.Ok, is GetAccountDataResult.RpcError -> {
                        daemon.setAccount(accountToken)
                        true
                    }
                    else -> false
                }
            }

            if (loginSucceeded) {
                loggedIn("")
            } else {
                loginFailure(R.string.login_fail_description)
            }
        }
    }

    private suspend fun loggedIn(subtitleMessage: String) {
        showLoggedInMessage(subtitleMessage)
        delay(1000)
        loggedIn.complete(Unit)
    }

    private fun showLoggedInMessage(subtitleMessage: String) {
        title.setText(R.string.logged_in_title)
        subtitle.setText(subtitleMessage)

        loggingInStatus.visibility = View.GONE
        loginFailStatus.visibility = View.GONE
        loggedInStatus.visibility = View.VISIBLE

        accountInput.state = LoginState.Success
    }

    private fun openConnectScreen() {
        fragmentManager?.beginTransaction()?.apply {
            replace(R.id.main_fragment, ConnectFragment())
            commit()
        }
    }

    private fun loginFailure(description: Int) {
        title.setText(R.string.login_fail_title)
        subtitle.setText(description)

        loggingInStatus.visibility = View.GONE
        loginFailStatus.visibility = View.VISIBLE
        loggedInStatus.visibility = View.GONE

        accountInput.state = LoginState.Failure
    }
}
