package net.mullvad.mullvadvpn.ui.widget

import android.content.Context
import android.util.AttributeSet
import android.view.LayoutInflater
import android.view.View
import android.widget.ImageButton
import android.widget.LinearLayout
import android.widget.TextView
import kotlin.properties.Delegates.observable
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.ui.LoginState

const val MIN_ACCOUNT_TOKEN_LENGTH = 10

class AccountInput : LinearLayout {
    private val container =
        context.getSystemService(Context.LAYOUT_INFLATER_SERVICE).let { service ->
            val inflater = service as LayoutInflater

            inflater.inflate(R.layout.account_input, this)
        }

    private val input = container.findViewById<TextView>(R.id.login_input)

    private val button = container.findViewById<ImageButton>(R.id.login_button).apply {
        setOnClickListener {
            onLogin?.invoke(input.text.toString())
        }
    }

    var loginState by observable(LoginState.Initial) { _, _, state ->
        when (state) {
            LoginState.Initial -> initialState()
            LoginState.InProgress -> loggingInState()
            LoginState.Success -> successState()
            LoginState.Failure -> failureState()
        }
    }

    var onLogin: ((String) -> Unit)? = null

    constructor(context: Context) : super(context) {}

    constructor(context: Context, attributes: AttributeSet) : super(context, attributes) {}

    constructor(context: Context, attributes: AttributeSet, defaultStyleAttribute: Int) :
        super(context, attributes, defaultStyleAttribute) {}

    constructor(
        context: Context,
        attributes: AttributeSet,
        defaultStyleAttribute: Int,
        defaultStyleResource: Int
    ) : super(context, attributes, defaultStyleAttribute, defaultStyleResource) {
    }

    init {
        orientation = HORIZONTAL

        setButtonEnabled(false)
    }

    private fun initialState() {
        button.visibility = View.VISIBLE
        setButtonEnabled(input.text.length >= MIN_ACCOUNT_TOKEN_LENGTH)
    }

    private fun loggingInState() {
        button.visibility = View.GONE
        setButtonEnabled(false)
    }

    private fun successState() {
        button.visibility = View.GONE
        setButtonEnabled(false)
    }

    private fun failureState() {
        button.visibility = View.VISIBLE
        setButtonEnabled(false)
    }

    /*private*/ fun setButtonEnabled(enabled: Boolean) {
        button.apply {
            if (enabled != isEnabled()) {
                setEnabled(enabled)
                setClickable(enabled)
                setFocusable(enabled)
            }
        }
    }
}
