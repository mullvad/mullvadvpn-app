package net.mullvad.mullvadvpn

import kotlinx.coroutines.async
import kotlinx.coroutines.launch
import kotlinx.coroutines.Deferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.Job

import android.os.Bundle
import android.os.Handler
import android.support.v4.app.Fragment
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.TextView

class LoginFragment : Fragment() {
    private lateinit var title: TextView
    private lateinit var subtitle: TextView
    private lateinit var loggingInStatus: View
    private lateinit var loggedInStatus: View
    private lateinit var loginFailStatus: View
    private lateinit var accountInput: AccountInput

    private var loginJob: Deferred<Boolean>? = null

    override fun onCreateView(
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

        accountInput = AccountInput(view, context!!)
        accountInput.onLogin = { accountToken -> login(accountToken) }

        return view
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

    private fun performLogin(accountToken: String) = GlobalScope.launch(Dispatchers.Main) {
        loginJob?.cancel()
        loginJob = GlobalScope.async(Dispatchers.Default) {
            val parentActivity = activity as MainActivity
            val daemon = parentActivity.asyncDaemon.await()
            val accountData = daemon.getAccountData(accountToken)

            if (accountData != null) {
                true
            } else {
                false
            }
        }

        if (loginJob?.await() ?: false) {
            loggedIn()
        } else {
            loginFailure()
        }
    }

    private fun loggedIn() {
        title.setText(R.string.logged_in_title)
        subtitle.setText("")

        loggingInStatus.visibility = View.GONE
        loginFailStatus.visibility = View.GONE
        loggedInStatus.visibility = View.VISIBLE

        accountInput.state = LoginState.Success

        Handler().postDelayed(Runnable { openConnectScreen() }, 1000)
    }

    private fun openConnectScreen() {
        fragmentManager?.beginTransaction()?.apply {
            replace(R.id.main_fragment, ConnectFragment())
            commit()
        }
    }

    private fun loginFailure() {
        title.setText(R.string.login_fail_title)
        subtitle.setText(R.string.login_fail_description)

        loggingInStatus.visibility = View.GONE
        loginFailStatus.visibility = View.VISIBLE
        loggedInStatus.visibility = View.GONE

        accountInput.state = LoginState.Failure
    }
}
