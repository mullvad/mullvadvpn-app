package net.mullvad.mullvadvpn

import android.content.Context
import android.view.View
import android.text.Editable
import android.text.TextWatcher
import android.widget.EditText
import android.widget.ImageButton

const val MIN_ACCOUNT_TOKEN_LENGTH = 10

class AccountInput(val parentView: View, val context: Context) {
    private val disabledBackgroundColor = context.getColor(R.color.white20)
    private val disabledTextColor = context.getColor(R.color.white)
    private val enabledBackgroundColor = context.getColor(R.color.white)
    private val enabledTextColor = context.getColor(R.color.blue)
    private val errorTextColor = context.getColor(R.color.red)

    private var usingErrorColor = false

    var state = LoginState.Initial
        set(value) {
            when (value) {
                LoginState.Initial -> initialState()
                LoginState.InProgress -> loggingInState()
                LoginState.Success -> successState()
                LoginState.Failure -> failureState()
            }
        }

    val input: EditText = parentView.findViewById(R.id.account_input)
    val button: ImageButton = parentView.findViewById(R.id.login_button)

    var onLogin: ((String) -> Unit)? = null

    init {
        input.addTextChangedListener(InputWatcher())
        button.setOnClickListener { onLogin?.invoke(input.text.toString()) }
        setButtonEnabled(false)
    }

    private fun initialState() {
        setButtonEnabled(input.text.length >= MIN_ACCOUNT_TOKEN_LENGTH)
        button.visibility = View.VISIBLE

        input.apply {
            setBackgroundColor(enabledBackgroundColor)
            setTextColor(enabledTextColor)
            setEnabled(true)
            visibility = View.VISIBLE
        }
    }

    private fun loggingInState() {
        setButtonEnabled(false)
        button.visibility = View.GONE

        input.apply {
            setBackgroundColor(disabledBackgroundColor)
            setTextColor(disabledTextColor)
            setEnabled(false)
            visibility = View.VISIBLE
        }
    }

    private fun successState() {
        setButtonEnabled(false)
        button.visibility = View.GONE
        input.visibility = View.GONE
    }

    private fun failureState() {
        setButtonEnabled(false)
        button.visibility = View.VISIBLE

        input.apply {
            setBackgroundColor(enabledBackgroundColor)
            setTextColor(errorTextColor)
            setEnabled(true)
            visibility = View.VISIBLE
        }

        usingErrorColor = true
    }

    private fun setButtonEnabled(enabled: Boolean) {
        button.apply {
            if (enabled != isEnabled()) {
                setEnabled(enabled)
                setClickable(enabled)
                setFocusable(enabled)
            }
        }
    }

    inner class InputWatcher : TextWatcher {
        override fun beforeTextChanged(text: CharSequence, start: Int, count: Int, after: Int) {}

        override fun onTextChanged(text: CharSequence, start: Int, before: Int, count: Int) {}

        override fun afterTextChanged(text: Editable) {
            setButtonEnabled(text.length >= MIN_ACCOUNT_TOKEN_LENGTH)

            if (usingErrorColor) {
                input.setTextColor(enabledTextColor)
                usingErrorColor = false
            }
        }
    }
}
