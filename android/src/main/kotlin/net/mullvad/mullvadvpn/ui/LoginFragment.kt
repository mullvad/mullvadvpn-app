package net.mullvad.mullvadvpn.ui

import android.content.Intent
import android.net.Uri
import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.TextView
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.Deferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.async
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.model.GetAccountDataResult

class LoginFragment : ServiceDependentFragment() {
    private lateinit var title: TextView
    private lateinit var subtitle: TextView
    private lateinit var loggingInStatus: View
    private lateinit var loggedInStatus: View
    private lateinit var loginFailStatus: View
    private lateinit var accountInput: AccountInput

    private val loggedIn = CompletableDeferred<Unit>()

    private var loginJob: Deferred<Boolean>? = null
    private var advanceToNextScreenJob: Job? = null

    override fun onCreateView(
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

        view.findViewById<View>(R.id.create_account).setOnClickListener { createAccount() }

        return view
    }

    override fun onResume() {
        super.onResume()
        advanceToNextScreenJob = GlobalScope.launch(Dispatchers.Main) {
            loggedIn.join()
            openConnectScreen()
        }
    }

    override fun onPause() {
        advanceToNextScreenJob?.cancel()
        super.onPause()
    }

    private fun createAccount() {
        val uri = Uri.parse(parentActivity.getString(R.string.create_account_url))
        val intent = Intent(Intent.ACTION_VIEW, uri)

        startActivity(intent)
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
            val accountDataResult = daemon.getAccountData(accountToken)

            when (accountDataResult) {
                is GetAccountDataResult.Ok, is GetAccountDataResult.RpcError -> {
                    daemon.setAccount(accountToken)
                    true
                }
                else -> false
            }
        }

        if (loginJob?.await() ?: false) {
            loggedIn()
        } else {
            loginFailure()
        }
    }

    private suspend fun loggedIn() {
        showLoggedInMessage()
        delay(1000)
        loggedIn.complete(Unit)
    }

    private fun showLoggedInMessage() {
        title.setText(R.string.logged_in_title)
        subtitle.setText("")

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

    private fun loginFailure() {
        title.setText(R.string.login_fail_title)
        subtitle.setText(R.string.login_fail_description)

        loggingInStatus.visibility = View.GONE
        loginFailStatus.visibility = View.VISIBLE
        loggedInStatus.visibility = View.GONE

        accountInput.state = LoginState.Failure
    }
}
