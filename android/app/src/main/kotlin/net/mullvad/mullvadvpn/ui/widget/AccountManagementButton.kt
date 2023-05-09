package net.mullvad.mullvadvpn.ui.widget

import android.content.Context
import android.util.AttributeSet
import net.mullvad.mullvadvpn.R

class AccountManagementButton : UrlButton {
    constructor(context: Context) : super(context)

    constructor(context: Context, attributes: AttributeSet) : super(context, attributes)

    constructor(
        context: Context,
        attributes: AttributeSet,
        defaultStyleAttribute: Int
    ) : super(context, attributes, defaultStyleAttribute)

    init {
        label = context.getString(R.string.manage_account)
    }
}
