package net.mullvad.mullvadvpn.ui.widget

import android.content.Context
import android.util.AttributeSet
import android.view.LayoutInflater
import android.widget.ImageButton
import android.widget.LinearLayout
import android.widget.TextView
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.ui.LoginState

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

    var loginState = LoginState.Initial

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
    }
}
