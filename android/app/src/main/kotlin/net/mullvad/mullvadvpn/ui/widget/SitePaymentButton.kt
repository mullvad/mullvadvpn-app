package net.mullvad.mullvadvpn.ui.widget

import android.content.Context
import android.util.AttributeSet
import kotlin.properties.Delegates.observable
import net.mullvad.mullvadvpn.R

class SitePaymentButton : UrlButton {
    constructor(context: Context) : super(context)

    constructor(context: Context, attributes: AttributeSet) : super(context, attributes)

    constructor(
        context: Context,
        attributes: AttributeSet,
        defaultStyleAttribute: Int
    ) : super(context, attributes, defaultStyleAttribute)

    var newAccount by
        observable(false) { _, _, isNewAccount ->
            if (isNewAccount) {
                label = context.getString(R.string.buy_credit)
            } else {
                label = context.getString(R.string.buy_more_credit)
            }
        }
}
