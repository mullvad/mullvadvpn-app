package net.mullvad.mullvadvpn.ui

import android.content.Context
import android.view.View
import android.widget.ArrayAdapter
import android.widget.ListView
import kotlin.properties.Delegates.observable
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.ui.widget.AccountInput
import net.mullvad.mullvadvpn.ui.widget.AccountLoginBorder
import net.mullvad.mullvadvpn.ui.widget.AccountLoginBorder.BorderState

class AccountInputController(val parentView: View, context: Context) {
    private var inputHasFocus by observable(false) { _, _, hasFocus ->
        updateBorder()

        if (hasFocus) {
            shouldShowAccountHistory = true
        }
    }

    var state: LoginState by observable(LoginState.Initial) { _, _, newState ->
        input.loginState = newState

        updateBorder()

        when (newState) {
            LoginState.Initial -> {}
            LoginState.InProgress -> loggingInState()
            LoginState.Success -> successState()
            LoginState.Failure -> {}
        }
    }

    val container: AccountLoginBorder = parentView.findViewById(R.id.account_input_container)
    val accountHistoryList: ListView = parentView.findViewById(R.id.account_history_list)

    val input = parentView.findViewById<AccountInput>(R.id.account_input).apply {
        onFocusChanged.subscribe(this) { hasFocus ->
            inputHasFocus = hasFocus
        }

        onTextChanged.subscribe(this) { _ ->
            if (state == LoginState.Failure) {
                state = LoginState.Initial
            }
        }
    }

    var accountHistory: ArrayList<String>? = null
        set(value) {
            synchronized(this) {
                field = value
                updateAccountHistory()
            }
        }

    private var shouldShowAccountHistory = false
        set(value) {
            synchronized(this) {
                field = value
                updateAccountHistory()
            }
        }

    var onLogin: ((String) -> Unit)?
        get() = input.onLogin
        set(value) { input.onLogin = value }

    fun onDestroy() {
        input.onFocusChanged.unsubscribe(this)
        input.onTextChanged.unsubscribe(this)
    }

    private fun loggingInState() {
        accountHistoryList.visibility = View.INVISIBLE
    }

    private fun successState() {
        container.visibility = View.INVISIBLE
    }

    private fun updateAccountHistory() {
        accountHistory?.let { history ->
            accountHistoryList.apply {
                setAdapter(
                    ArrayAdapter(
                        context,
                        R.layout.account_history_entry,
                        R.id.account_history_entry_text_view,
                        history
                    )
                )

                setOnItemClickListener { _, _, idx, _ ->
                    input.loginWith(history[idx])
                    accountHistoryList.visibility = View.GONE
                }
            }

            if (shouldShowAccountHistory && accountHistoryList.visibility != View.VISIBLE) {
                accountHistoryList.visibility = View.VISIBLE
                accountHistoryList.translationY = -accountHistoryList.height.toFloat()
                accountHistoryList.animate().translationY(0.0F).setDuration(350).start()
            }
        }
    }

    private fun updateBorder() {
        if (state == LoginState.Failure) {
            container.borderState = BorderState.ERROR
        } else if (inputHasFocus) {
            container.borderState = BorderState.FOCUSED
        } else {
            container.borderState = BorderState.UNFOCUSED
        }
    }
}
