package net.mullvad.mullvadvpn.ui

import android.content.Context
import android.text.Editable
import android.text.TextWatcher
import android.view.MotionEvent
import android.view.View
import android.view.View.OnTouchListener
import android.widget.ArrayAdapter
import android.widget.ListView
import android.widget.TextView
import kotlin.properties.Delegates.observable
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.ui.AccountInputContainer.BorderState
import net.mullvad.mullvadvpn.ui.widget.AccountInput

class AccountInputController(val parentView: View, context: Context) {
    private val disabledBackgroundColor = context.getColor(R.color.white20)
    private val disabledTextColor = context.getColor(R.color.white)
    private val enabledBackgroundColor = context.getColor(R.color.white)
    private val enabledTextColor = context.getColor(R.color.blue)
    private val errorTextColor = context.getColor(R.color.red)

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
            LoginState.Initial -> initialState()
            LoginState.InProgress -> loggingInState()
            LoginState.Success -> successState()
            LoginState.Failure -> failureState()
        }
    }

    val container: AccountInputContainer = parentView.findViewById(R.id.account_input_container)
    val input: TextView = parentView.findViewById(R.id.login_input)
    val accountHistoryList: ListView = parentView.findViewById(R.id.account_history_list)

    val newInput = parentView.findViewById<AccountInput>(R.id.account_input)

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
            setOnTouchListener(
                OnTouchListener {
                    _, event ->
                    if (MotionEvent.ACTION_UP == event.getAction()) {
                        shouldShowAccountHistory = true
                    }
                    false
                }
            )
        }

        container.setOnClickListener { shouldShowAccountHistory = true }
    }

    private fun initialState() {
        input.apply {
            setTextColor(enabledTextColor)
            setEnabled(true)
            visibility = View.VISIBLE
        }
    }

    private fun loggingInState() {
        input.apply {
            setTextColor(disabledTextColor)
            setEnabled(false)
            visibility = View.VISIBLE
            clearFocus()
        }
        accountHistoryList.visibility = View.INVISIBLE
    }

    private fun successState() {
        input.visibility = View.GONE
        container.visibility = View.INVISIBLE
    }

    private fun failureState() {
        input.apply {
            findFocus()
            setTextColor(errorTextColor)
            setEnabled(true)
            visibility = View.VISIBLE
        }

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
                    val accountNumber = history[idx]

                    input.setText(accountNumber)
                    accountHistoryList.visibility = View.GONE
                    onLogin?.invoke(accountNumber)
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
            input.setTextColor(enabledTextColor)
            usingErrorColor = false
        }
    }

    inner class InputWatcher : TextWatcher {
        override fun beforeTextChanged(text: CharSequence, start: Int, count: Int, after: Int) {}

        override fun onTextChanged(text: CharSequence, start: Int, before: Int, count: Int) {}

        override fun afterTextChanged(text: Editable) {
            inputHasFocus = true
            leaveErrorState()
        }
    }
}
