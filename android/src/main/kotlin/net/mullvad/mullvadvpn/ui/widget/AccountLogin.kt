package net.mullvad.mullvadvpn.ui.widget

import android.content.Context
import android.util.AttributeSet
import android.view.LayoutInflater
import android.view.View
import android.widget.ArrayAdapter
import android.widget.ListView
import android.widget.RelativeLayout
import kotlin.properties.Delegates.observable
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.ui.LoginState
import net.mullvad.mullvadvpn.ui.widget.AccountLoginBorder.BorderState

class AccountLogin : RelativeLayout {
    private val container =
        context.getSystemService(Context.LAYOUT_INFLATER_SERVICE).let { service ->
            val inflater = service as LayoutInflater

            inflater.inflate(R.layout.account_login, this)
        }

    private val border: AccountLoginBorder = container.findViewById(R.id.border)
    private val accountHistoryList: ListView = container.findViewById(R.id.history)
    private val input: AccountInput = container.findViewById(R.id.input)

    private var shouldShowAccountHistory = false
        set(value) {
            synchronized(this) {
                field = value
                updateAccountHistory()
            }
        }

    private var inputHasFocus by observable(false) { _, _, hasFocus ->
        updateBorder()

        if (hasFocus) {
            shouldShowAccountHistory = true
        }
    }

    var accountHistory: ArrayList<String>? = null
        set(value) {
            synchronized(this) {
                field = value
                updateAccountHistory()
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

    var onLogin: ((String) -> Unit)?
        get() = input.onLogin
        set(value) { input.onLogin = value }

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
        border.elevation = elevation + 0.1f

        input.apply {
            onFocusChanged.subscribe(this) { hasFocus ->
                inputHasFocus = hasFocus
            }

            onTextChanged.subscribe(this) { _ ->
                if (state == LoginState.Failure) {
                    state = LoginState.Initial
                }
            }
        }
    }

    fun onDestroy() {
        input.onFocusChanged.unsubscribe(this)
        input.onTextChanged.unsubscribe(this)
    }

    private fun loggingInState() {
        accountHistoryList.visibility = View.INVISIBLE
    }

    private fun successState() {
        visibility = View.INVISIBLE
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
            border.borderState = BorderState.ERROR
        } else if (inputHasFocus) {
            border.borderState = BorderState.FOCUSED
        } else {
            border.borderState = BorderState.UNFOCUSED
        }
    }
}
