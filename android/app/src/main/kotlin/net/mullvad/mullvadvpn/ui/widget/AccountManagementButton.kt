package net.mullvad.mullvadvpn.ui.widget

import android.content.Context
import android.util.AttributeSet
import kotlin.properties.Delegates.observable
import net.mullvad.mullvadvpn.BuildConfig
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.constant.BuildTypes

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
