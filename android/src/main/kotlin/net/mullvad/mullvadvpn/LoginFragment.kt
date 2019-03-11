package net.mullvad.mullvadvpn

import android.os.Bundle
import android.support.v4.app.Fragment
import android.text.Editable
import android.text.TextWatcher
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.EditText
import android.widget.ImageButton

const val MIN_ACCOUNT_TOKEN_LENGTH = 10

class LoginFragment : Fragment() {
    private lateinit var loginButton: ImageButton

    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View {
        val view = inflater.inflate(R.layout.login, container, false)
        val accountInput: EditText = view.findViewById(R.id.account_input)

        loginButton = view.findViewById(R.id.login_button)

        accountInput.addTextChangedListener(AccountInputWatcher())
        setLoginButtonEnabled(false)

        return view
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

    inner class AccountInputWatcher : TextWatcher {
        override fun beforeTextChanged(text: CharSequence, start: Int, count: Int, after: Int) {}

        override fun onTextChanged(text: CharSequence, start: Int, before: Int, count: Int) {}

        override fun afterTextChanged(text: Editable) {
            setLoginButtonEnabled(text.length >= MIN_ACCOUNT_TOKEN_LENGTH)
        }
    }
}
