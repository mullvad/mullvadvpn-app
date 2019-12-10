package net.mullvad.mullvadvpn.ui

import android.content.Context
import android.graphics.Outline
import android.view.View
import android.view.ViewOutlineProvider
import net.mullvad.mullvadvpn.R

class AccountInputOutlineProvider(private val context: Context) : ViewOutlineProvider() {
    private val cornerRadius = context.resources.getDimension(R.dimen.account_input_corner_radius)

    override fun getOutline(view: View, outline: Outline) {
        outline.setRoundRect(0, 0, view.width, view.height, cornerRadius)
    }
}
