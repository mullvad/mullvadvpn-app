package net.mullvad.mullvadvpn.ui.widget

import android.content.Context
import android.text.Editable
import android.text.TextWatcher
import android.text.method.DigitsKeyListener
import android.text.style.MetricAffectingSpan
import android.util.AttributeSet
import android.view.LayoutInflater
import android.view.View
import android.view.View.OnFocusChangeListener
import android.widget.EditText
import android.widget.ImageButton
import android.widget.LinearLayout
import kotlin.properties.Delegates.observable
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.ui.LoginState
import net.mullvad.mullvadvpn.util.SegmentedInputFormatter
import net.mullvad.mullvadvpn.util.setOnEnterOrDoneAction
import net.mullvad.talpid.util.EventNotifier

const val MIN_ACCOUNT_TOKEN_LENGTH = 10

class AccountInput : LinearLayout {
    private val disabledTextColor = context.getColor(R.color.white)
    private val enabledTextColor = context.getColor(R.color.blue)
    private val errorTextColor = context.getColor(R.color.red)

    private val container =
        context.getSystemService(Context.LAYOUT_INFLATER_SERVICE).let { service ->
            val inflater = service as LayoutInflater

            inflater.inflate(R.layout.account_input, this)
        }

    private val inputWatcher = object : TextWatcher {
        override fun beforeTextChanged(text: CharSequence, start: Int, count: Int, after: Int) {}

        override fun onTextChanged(text: CharSequence, start: Int, before: Int, count: Int) {}

        override fun afterTextChanged(text: Editable) {
            removeFormattingSpans(text)
            setButtonEnabled(text.length >= MIN_ACCOUNT_TOKEN_LENGTH)
            onTextChanged.notify(text.toString())
        }
    }

    val input = container.findViewById<EditText>(R.id.login_input).apply {
        addTextChangedListener(inputWatcher)
        setOnEnterOrDoneAction(::login)

        onFocusChangeListener = OnFocusChangeListener { view, inputHasFocus ->
            hasFocus = inputHasFocus && view.isEnabled
        }

        // Manually initializing the `DigitsKeyListener` allows spaces to be used and still keeps
        // the input type as a number so that the correct software keyboard type is shown
        keyListener = DigitsKeyListener.getInstance("01234567890 ")

        SegmentedInputFormatter(this, ' ').apply {
            isValidInputCharacter = { character ->
                '0' <= character && character <= '9'
            }
        }
    }

    private val button = container.findViewById<ImageButton>(R.id.login_button).apply {
        setOnClickListener { login() }
    }

    val onFocusChanged = EventNotifier(false)
    private var hasFocus by onFocusChanged.notifiable()

    val onTextChanged = EventNotifier("")

    var loginState by observable(LoginState.Initial) { _, _, state ->
        when (state) {
            LoginState.Initial -> initialState()
            LoginState.InProgress -> loggingInState()
            LoginState.Success -> successState()
            LoginState.Failure -> failureState()
        }
    }

    var onLogin: ((String) -> Unit)? = null

    constructor(context: Context) : super(context)

    constructor(context: Context, attributes: AttributeSet) : super(context, attributes)

    constructor(context: Context, attributes: AttributeSet, defaultStyleAttribute: Int) :
        super(context, attributes, defaultStyleAttribute)

    init {
        orientation = HORIZONTAL

        setButtonEnabled(false)
    }

    fun loginWith(accountNumber: String) {
        input.setText(accountNumber)
        onLogin?.invoke(accountNumber)
    }

    private fun login() {
        onLogin?.invoke(input.text.replace(Regex("[^0-9]"), ""))
    }

    private fun initialState() {
        input.apply {
            setTextColor(enabledTextColor)
            setEnabled(true)
            setFocusableInTouchMode(true)
            visibility = View.VISIBLE
        }

        button.visibility = View.VISIBLE
        setButtonEnabled(input.text.length >= MIN_ACCOUNT_TOKEN_LENGTH)
    }

    private fun loggingInState() {
        input.apply {
            setTextColor(disabledTextColor)
            setEnabled(false)
            setFocusable(false)
            visibility = View.VISIBLE
        }

        button.visibility = View.GONE
        setButtonEnabled(false)
    }

    private fun successState() {
        input.apply {
            setTextColor(disabledTextColor)
            setEnabled(false)
            setFocusable(false)
            visibility = View.VISIBLE
        }

        button.visibility = View.GONE
        setButtonEnabled(false)
    }

    private fun failureState() {
        button.visibility = View.VISIBLE
        setButtonEnabled(true)

        input.apply {
            setTextColor(errorTextColor)
            setEnabled(true)
            setFocusableInTouchMode(true)
            visibility = View.VISIBLE
            requestFocus()
        }
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

    private fun removeFormattingSpans(text: Editable) {
        for (span in text.getSpans(0, text.length, MetricAffectingSpan::class.java)) {
            text.removeSpan(span)
        }
    }
}
