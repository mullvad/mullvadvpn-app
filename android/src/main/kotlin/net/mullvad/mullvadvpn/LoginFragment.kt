package net.mullvad.mullvadvpn

import android.content.Context
import android.os.Bundle
import android.support.v4.app.Fragment
import android.text.Editable
import android.text.TextWatcher
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.EditText
import android.widget.ImageButton
import android.widget.TextView

const val MIN_ACCOUNT_TOKEN_LENGTH = 10

class LoginFragment : Fragment() {
    private lateinit var title: TextView
    private lateinit var subtitle: TextView
    private lateinit var statusIcon: View
    private lateinit var accountInput: EditText
    private lateinit var loginButton: ImageButton

    private var accountInputDisabledBackgroundColor: Int = 0
    private var accountInputDisabledTextColor: Int = 0

    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View {
        val view = inflater.inflate(R.layout.login, container, false)

        title = view.findViewById(R.id.title)
        subtitle = view.findViewById(R.id.subtitle)
        statusIcon = view.findViewById(R.id.status_icon)
        accountInput = view.findViewById(R.id.account_input)
        loginButton = view.findViewById(R.id.login_button)

        accountInput.addTextChangedListener(AccountInputWatcher())
        loginButton.setOnClickListener { login() }
        setLoginButtonEnabled(false)

        return view
    }

    override fun onAttach(context: Context) {
        super.onAttach(context)

        accountInputDisabledBackgroundColor = context.getColor(R.color.white20)
        accountInputDisabledTextColor = context.getColor(R.color.white)
    }

    private fun setLoginButtonEnabled(enabled: Boolean) {
        loginButton.apply {
            if (enabled != isEnabled()) {
                setEnabled(enabled)
                setClickable(enabled)
                setFocusable(enabled)
            }
        }
    }

    private fun login() {
        title.setText(R.string.logging_in_title)
        subtitle.setText(R.string.logging_in_description)
        statusIcon.setVisibility(View.VISIBLE)
        loginButton.setVisibility(View.GONE)
        disableAccountInput()
    }

    private fun disableAccountInput() {
        accountInput.apply {
            setEnabled(false)
            setBackgroundColor(accountInputDisabledBackgroundColor)
            setTextColor(accountInputDisabledTextColor)
        }
    }

    inner class AccountInputWatcher : TextWatcher {
        override fun beforeTextChanged(text: CharSequence, start: Int, count: Int, after: Int) {}

        override fun onTextChanged(text: CharSequence, start: Int, before: Int, count: Int) {}

        override fun afterTextChanged(text: Editable) {
            setLoginButtonEnabled(text.length >= MIN_ACCOUNT_TOKEN_LENGTH)
        }
    }
}
