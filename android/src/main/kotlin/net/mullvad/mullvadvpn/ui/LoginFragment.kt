package net.mullvad.mullvadvpn.ui

import android.graphics.Rect
import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.ScrollView
import android.widget.TextView
import androidx.core.content.ContextCompat
import kotlinx.coroutines.delay
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.model.LoginStatus
import net.mullvad.mullvadvpn.ui.widget.AccountLogin
import net.mullvad.mullvadvpn.ui.widget.Button

class LoginFragment : ServiceDependentFragment(OnNoService.GoToLaunchScreen), NavigationBarPainter {
    companion object {
        private enum class State {
            Starting,
            Idle,
            LoggingIn,
            CreatingAccount,
        }
    }

    private lateinit var title: TextView
    private lateinit var subtitle: TextView
    private lateinit var loggingInStatus: View
    private lateinit var loggedInStatus: View
    private lateinit var loginFailStatus: View
    private lateinit var accountLogin: AccountLogin
    private lateinit var scrollArea: ScrollView
    private lateinit var background: View

    private var loginStatus: LoginStatus? = null
    private var state = State.Starting

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
            onClearHistory = { -> accountCache.clearAccountHistory() }
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

        accountCache.onAccountHistoryChange.subscribe(this) { history ->
            jobTracker.newUiJob("updateHistory") {
                accountLogin.accountHistory = history
            }
        }

        accountCache.onLoginStatusChange.subscribe(this) { status ->
            jobTracker.newUiJob("updateLoginStatus") {
                loginStatus = status

                if (status == null) {
                    if (state == State.LoggingIn || state == State.CreatingAccount) {
                        loginFailure()
                    }
                } else {
                    if (state == State.Starting) {
                        openNextScreen()
                    } else {
                        loggedIn()
                    }
                }
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

        state = State.Idle
    }

    override fun onResume() {
        super.onResume()
        paintNavigationBar(ContextCompat.getColor(requireContext(), R.color.darkBlue))
    }

    override fun onSafelyStop() {
        jobTracker.cancelJob("advanceToNextScreen")
        accountCache.onAccountHistoryChange.unsubscribe(this)
        accountCache.onLoginStatusChange.unsubscribe(this)
        parentActivity.backButtonHandler = null
    }

    private fun scrollToShow(view: View) {
        val rectangle = Rect(0, 0, view.width, view.height)

        scrollArea.requestChildRectangleOnScreen(view, rectangle, false)
    }

    private suspend fun createAccount() {
        state = State.CreatingAccount

        title.setText(R.string.logging_in_title)
        subtitle.setText(R.string.creating_new_account)

        loggingInStatus.visibility = View.VISIBLE
        loginFailStatus.visibility = View.GONE
        loggedInStatus.visibility = View.GONE

        accountLogin.state = LoginState.InProgress

        scrollToShow(loggingInStatus)

        accountCache.createNewAccount()
    }

    private fun login(accountToken: String) {
        state = State.LoggingIn

        title.setText(R.string.logging_in_title)
        subtitle.setText(R.string.logging_in_description)

        loggingInStatus.visibility = View.VISIBLE
        loginFailStatus.visibility = View.GONE
        loggedInStatus.visibility = View.GONE

        background.requestFocus()

        accountLogin.state = LoginState.InProgress

        scrollToShow(loggingInStatus)

        accountCache.login(accountToken)
    }

    private suspend fun loggedIn() {
        if (loginStatus?.isNewAccount ?: false) {
            showLoggedInMessage(resources.getString(R.string.account_created))
        } else {
            showLoggedInMessage("")
        }

        delay(1000)
        openNextScreen()
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

    private fun openNextScreen() {
        val status = loginStatus

        val fragment = when {
            status == null -> return
            status.isNewAccount -> WelcomeFragment()
            status.isExpired -> OutOfTimeFragment()
            else -> ConnectFragment()
        }

        parentFragmentManager.beginTransaction().apply {
            replace(R.id.main_fragment, fragment)
            commit()
        }
    }

    private fun loginFailure() {
        val description = when (state) {
            State.LoggingIn -> R.string.login_fail_description
            State.CreatingAccount -> R.string.failed_to_create_account
            State.Idle, State.Starting -> return
        }

        state = State.Idle

        title.setText(R.string.login_fail_title)
        subtitle.setText(description)

        loggingInStatus.visibility = View.GONE
        loginFailStatus.visibility = View.VISIBLE
        loggedInStatus.visibility = View.GONE

        accountLogin.state = LoginState.Failure

        scrollToShow(accountLogin)
    }
}
