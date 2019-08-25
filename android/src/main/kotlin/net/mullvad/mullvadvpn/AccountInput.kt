package net.mullvad.mullvadvpn

import android.content.Context
import android.view.View
import android.view.View.OnFocusChangeListener
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

    private val resources = context.resources
    private val focusedBorder = resources.getDrawable(R.drawable.account_input_border_focused, null)
    private val errorBorder = resources.getDrawable(R.drawable.account_input_border_error, null)

    private var inputHasFocus = false
        set(value) {
            field = value
            updateBorder()
        }
    private var usingErrorColor = false
        set(value) {
            field = value
            updateBorder()
        }

    var state = LoginState.Initial
        set(value) {
            when (value) {
                LoginState.Initial -> initialState()
                LoginState.InProgress -> loggingInState()
                LoginState.Success -> successState()
                LoginState.Failure -> failureState()
            }
        }

    val container: View = parentView.findViewById(R.id.account_input_container)
    val input: EditText = parentView.findViewById(R.id.account_input)
    val button: ImageButton = parentView.findViewById(R.id.login_button)

    var onLogin: ((String) -> Unit)? = null

    init {
        button.setOnClickListener { onLogin?.invoke(input.text.toString()) }
        setButtonEnabled(false)

        input.apply {
            addTextChangedListener(InputWatcher())
            onFocusChangeListener = OnFocusChangeListener { view, hasFocus ->
                inputHasFocus = hasFocus && view.isEnabled()
            }
        }

        container.apply {
            clipToOutline = true
            outlineProvider = AccountInputOutlineProvider(context)
        }
    }

    private fun initialState() {
        setButtonEnabled(input.text.length >= MIN_ACCOUNT_TOKEN_LENGTH)
        button.visibility = View.VISIBLE

        input.apply {
            setTextColor(enabledTextColor)
            setEnabled(true)
            visibility = View.VISIBLE
        }
    }

    private fun loggingInState() {
        setButtonEnabled(false)
        button.visibility = View.GONE

        input.apply {
            setTextColor(disabledTextColor)
            setEnabled(false)
            visibility = View.VISIBLE
            clearFocus()
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
            findFocus()
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

    private fun updateBorder() {
        if (usingErrorColor) {
            container.foreground = errorBorder
        } else {
            if (inputHasFocus) {
                container.foreground = focusedBorder
            } else {
                container.foreground = null
            }
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
            setButtonEnabled(text.length >= MIN_ACCOUNT_TOKEN_LENGTH)
            leaveErrorState()
        }
    }
}
