package net.mullvad.mullvadvpn.ui

import android.graphics.Rect
import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.ScrollView
import android.widget.TextView
import androidx.core.content.ContextCompat
import androidx.fragment.app.Fragment
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.delay
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.model.GetAccountDataResult
import net.mullvad.mullvadvpn.service.AccountCache
import net.mullvad.mullvadvpn.ui.widget.AccountLogin
import net.mullvad.mullvadvpn.ui.widget.Button
import org.joda.time.DateTime

class LoginFragment : ServiceDependentFragment(OnNoService.GoToLaunchScreen), NavigationBarPainter {
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
    private lateinit var accountLogin: AccountLogin
    private lateinit var scrollArea: ScrollView
    private lateinit var background: View

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

        accountLogin = view.findViewById<AccountLogin>(R.id.account_login).apply {
            onLogin = { accountToken -> login(accountToken) }
            onRemoveFromHistory = { account -> accountCache.removeAccountFromHistory(account) }
        }

        view.findViewById<Button>(R.id.create_account)
            .setOnClickAction("createAccount", jobTracker) { createAccount() }

        scrollArea = view.findViewById(R.id.scroll_area)

        background = view.findViewById<View>(R.id.contents).apply {
            setOnClickListener { requestFocus() }
        }

        scrollToShow(accountLogin)

        return view
    }

    override fun onSafelyStart() {
        accountLogin.state = LoginState.Initial

        jobTracker.newBackgroundJob("checkIfAlreadyLoggedIn") {
            if (accountCache.onAccountNumberChange.latestEvent != null) {
                val loginResult = if (accountCache.newlyCreatedAccount) {
                    LoginResult.NewAccount
                } else {
                    loginResultForExpiry(accountCache.onAccountExpiryChange.latestEvent)
                }

                loggedIn.complete(loginResult)
            }
        }

        jobTracker.newUiJob("advanceToNextScreen") {
            when (loggedIn.await()) {
                LoginResult.ExistingAccountWithTime -> openNextScreen(ConnectFragment())
                LoginResult.ExistingAccountOutOfTime -> openNextScreen(OutOfTimeFragment())
                LoginResult.NewAccount -> openNextScreen(WelcomeFragment())
            }
        }

        accountCache.onAccountHistoryChange.subscribe(this) { history ->
            jobTracker.newUiJob("updateHistory") {
                accountLogin.accountHistory = history
            }
        }

        parentActivity.backButtonHandler = {
            if (accountLogin.hasFocus) {
                background.requestFocus()
                true
            } else {
                false
            }
        }
    }

    override fun onResume() {
        super.onResume()
        paintNavigationBar(ContextCompat.getColor(requireContext(), R.color.darkBlue))
    }

    override fun onSafelyStop() {
        jobTracker.cancelJob("advanceToNextScreen")
        accountCache.onAccountHistoryChange.unsubscribe(this)
        parentActivity.backButtonHandler = null
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

        accountLogin.state = LoginState.InProgress

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

        background.requestFocus()

        accountLogin.state = LoginState.InProgress

        scrollToShow(loggingInStatus)

        performLogin(accountToken)
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

                        loginResultForExpiry(expiry)
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

        accountLogin.state = LoginState.Success

        scrollToShow(loggedInStatus)
    }

    private fun openNextScreen(fragment: Fragment) {
        parentFragmentManager.beginTransaction().apply {
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

        accountLogin.state = LoginState.Failure

        scrollToShow(accountLogin)
    }

    private fun loginResultForExpiry(expiry: DateTime?): LoginResult {
        if (expiry == null || expiry.isAfterNow()) {
            return LoginResult.ExistingAccountWithTime
        } else {
            return LoginResult.ExistingAccountOutOfTime
        }
    }
}
