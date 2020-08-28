package net.mullvad.mullvadvpn.ui

import android.graphics.Rect
import android.os.Bundle
import android.support.v4.app.Fragment
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.ScrollView
import android.widget.TextView
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.delay
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.model.GetAccountDataResult
import net.mullvad.mullvadvpn.service.AccountCache
import net.mullvad.mullvadvpn.ui.widget.Button
import org.joda.time.DateTime

class LoginFragment : ServiceDependentFragment(OnNoService.GoToLaunchScreen) {
    enum class LoginResult {
        ExistingAccountWithTime,
        ExistingAccountOutOfTime,
        NewAccount;
    }

    private lateinit var title: TextView
    private lateinit var subtitle: TextView
    private lateinit var loggingInStatus: View
    private lateinit var loggedInStatus: View
    private lateinit var loginFailStatus: View
    private lateinit var accountInput: AccountInputController
    private lateinit var scrollArea: ScrollView

    private val loggedIn = CompletableDeferred<LoginResult>()

    override fun onSafelyCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View {
        val view = inflater.inflate(R.layout.login, container, false)

        title = view.findViewById(R.id.title)
        subtitle = view.findViewById(R.id.subtitle)
        loggingInStatus = view.findViewById(R.id.logging_in_status)
        loggedInStatus = view.findViewById(R.id.logged_in_status)
        loginFailStatus = view.findViewById(R.id.login_fail_status)

        accountInput = AccountInputController(view, parentActivity)
        accountInput.onLogin = { accountToken -> login(accountToken) }

        view.findViewById<Button>(R.id.create_account)
            .setOnClickAction("createAccount", jobTracker) { createAccount() }

        scrollArea = view.findViewById(R.id.scroll_area)

        fetchHistory()
        scrollToShow(accountInput.input)

        return view
    }

    override fun onSafelyResume() {
        jobTracker.newUiJob("advanceToNextScreen") {
            when (loggedIn.await()) {
                LoginResult.ExistingAccountWithTime -> openNextScreen(ConnectFragment())
                LoginResult.ExistingAccountOutOfTime -> openNextScreen(OutOfTimeFragment())
                LoginResult.NewAccount -> openNextScreen(WelcomeFragment())
            }
        }

        fetchHistory()
    }

    override fun onSafelyPause() {
        jobTracker.cancelJob("advanceToNextScreen")
    }

    private fun scrollToShow(view: View) {
        val rectangle = Rect(0, 0, view.width, view.height)

        scrollArea.requestChildRectangleOnScreen(view, rectangle, false)
    }

    private suspend fun createAccount() {
        title.setText(R.string.logging_in_title)
        subtitle.setText(R.string.creating_new_account)

        loggingInStatus.visibility = View.VISIBLE
        loginFailStatus.visibility = View.GONE
        loggedInStatus.visibility = View.GONE

        accountInput.state = LoginState.InProgress

        scrollToShow(loggingInStatus)

        val accountToken = jobTracker.runOnBackground {
            accountCache.createNewAccount()
        }

        if (accountToken == null) {
            loginFailure(R.string.failed_to_create_account)
        } else {
            loggedIn(resources.getString(R.string.account_created), LoginResult.NewAccount)
        }
    }

    private fun login(accountToken: String) {
        title.setText(R.string.logging_in_title)
        subtitle.setText(R.string.logging_in_description)

        loggingInStatus.visibility = View.VISIBLE
        loginFailStatus.visibility = View.GONE
        loggedInStatus.visibility = View.GONE

        accountInput.state = LoginState.InProgress

        scrollToShow(loggingInStatus)

        performLogin(accountToken)
    }

    private fun fetchHistory() {
        jobTracker.newUiJob("fetchHistory") {
            accountInput.accountHistory = jobTracker.runOnBackground() {
                daemon.getAccountHistory()
            }
        }
    }

    private fun performLogin(accountToken: String) {
        jobTracker.newUiJob("login") {
            val loginResult = jobTracker.runOnBackground {
                val accountDataResult = daemon.getAccountData(accountToken)

                when (accountDataResult) {
                    is GetAccountDataResult.Ok -> {
                        accountCache.login(accountToken)

                        val expiryString = accountDataResult.accountData.expiry
                        val expiry = DateTime.parse(expiryString, AccountCache.EXPIRY_FORMAT)

                        if (expiry.isAfterNow()) {
                            LoginResult.ExistingAccountWithTime
                        } else {
                            LoginResult.ExistingAccountOutOfTime
                        }
                    }
                    is GetAccountDataResult.RpcError -> {
                        accountCache.login(accountToken)
                        LoginResult.ExistingAccountWithTime
                    }
                    else -> null
                }
            }

            if (loginResult != null) {
                loggedIn("", loginResult)
            } else {
                loginFailure(R.string.login_fail_description)
            }
        }
    }

    private suspend fun loggedIn(subtitleMessage: String, result: LoginResult) {
        showLoggedInMessage(subtitleMessage)
        delay(1000)
        loggedIn.complete(result)
    }

    private fun showLoggedInMessage(subtitleMessage: String) {
        title.setText(R.string.logged_in_title)
        subtitle.setText(subtitleMessage)

        loggingInStatus.visibility = View.GONE
        loginFailStatus.visibility = View.GONE
        loggedInStatus.visibility = View.VISIBLE

        accountInput.state = LoginState.Success

        scrollToShow(loggedInStatus)
    }

    private fun openNextScreen(fragment: Fragment) {
        fragmentManager?.beginTransaction()?.apply {
            replace(R.id.main_fragment, fragment)
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

        scrollToShow(accountInput.input)
    }
}
