package net.mullvad.mullvadvpn.ui

import android.content.Context
import android.text.Editable
import android.text.TextWatcher
import android.view.View
import android.widget.ArrayAdapter
import android.widget.ListView
import android.widget.TextView
import kotlin.properties.Delegates.observable
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.ui.AccountInputContainer.BorderState
import net.mullvad.mullvadvpn.ui.widget.AccountInput

class AccountInputController(val parentView: View, context: Context) {
    private var inputHasFocus by observable(false) { _, _, hasFocus ->
        updateBorder()

        if (hasFocus) {
            shouldShowAccountHistory = true
        }
    }

    private var usingErrorColor by observable(false) { _, _, _ ->
        updateBorder()
    }

    var state: LoginState by observable(LoginState.Initial) { _, _, newState ->
        newInput.loginState = newState

        when (newState) {
            LoginState.Initial -> {}
            LoginState.InProgress -> loggingInState()
            LoginState.Success -> successState()
            LoginState.Failure -> failureState()
        }
    }

    val container: AccountInputContainer = parentView.findViewById(R.id.account_input_container)
    val input: TextView = parentView.findViewById(R.id.login_input)
    val accountHistoryList: ListView = parentView.findViewById(R.id.account_history_list)

    val newInput = parentView.findViewById<AccountInput>(R.id.account_input).apply {
        onFocusChanged.subscribe(this) { hasFocus ->
            inputHasFocus = hasFocus
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
        get() = newInput.onLogin
        set(value) { newInput.onLogin = value }

    init {
        input.apply {
            addTextChangedListener(InputWatcher())
        }

        container.setOnClickListener { shouldShowAccountHistory = true }
    }

    fun onDestroy() {
        newInput.onFocusChanged.unsubscribe(this)
    }

    private fun loggingInState() {
        accountHistoryList.visibility = View.INVISIBLE
    }

    private fun successState() {
        container.visibility = View.INVISIBLE
    }

    private fun failureState() {
        usingErrorColor = true
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
                    newInput.loginWith(history[idx])
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
        if (usingErrorColor) {
            container.borderState = BorderState.ERROR
        } else if (inputHasFocus) {
            container.borderState = BorderState.FOCUSED
        } else {
            container.borderState = BorderState.UNFOCUSED
        }
    }

    private fun leaveErrorState() {
        if (usingErrorColor) {
            newInput.loginState = LoginState.Initial
            usingErrorColor = false
        }
    }

    inner class InputWatcher : TextWatcher {
        override fun beforeTextChanged(text: CharSequence, start: Int, count: Int, after: Int) {}

        override fun onTextChanged(text: CharSequence, start: Int, before: Int, count: Int) {}

        override fun afterTextChanged(text: Editable) {
            leaveErrorState()
        }
    }
}
