package net.mullvad.mullvadvpn.ui

import android.content.Context
import android.util.AttributeSet
import android.view.View
import android.widget.ScrollView
import androidx.coordinatorlayout.widget.CoordinatorLayout
import androidx.coordinatorlayout.widget.CoordinatorLayout.Behavior
import net.mullvad.mullvadvpn.R

class UnderNotificationBannerBehavior(context: Context, attributes: AttributeSet) :
    Behavior<ScrollView>(context, attributes) {
    override fun layoutDependsOn(parent: CoordinatorLayout, body: ScrollView, dependency: View) =
        dependency.id == R.id.notification_banner

    override fun onDependentViewChanged(
        parent: CoordinatorLayout,
        body: ScrollView,
        dependency: View
    ): Boolean {
        val newPaddingTop =
            if (dependency.visibility == View.VISIBLE) {
                dependency.height + dependency.translationY.toInt()
            } else {
                0
            }

        body.getChildAt(0).apply {
            if (paddingTop != newPaddingTop) {
                setPadding(paddingLeft, newPaddingTop, paddingRight, paddingBottom)
                return true
            } else {
                return false
            }
        }
    }
}
