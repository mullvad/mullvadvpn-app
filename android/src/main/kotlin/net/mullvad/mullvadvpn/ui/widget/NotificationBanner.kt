package net.mullvad.mullvadvpn.ui.widget

import android.content.Context
import android.util.AttributeSet
import android.view.LayoutInflater
import android.view.View
import android.widget.FrameLayout
import android.widget.ImageView
import android.widget.TextView
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.ui.notification.InAppNotification
import net.mullvad.mullvadvpn.ui.notification.InAppNotificationController
import net.mullvad.mullvadvpn.ui.notification.StatusLevel
import net.mullvad.mullvadvpn.util.JobTracker

class NotificationBanner : FrameLayout {
    private val jobTracker = JobTracker()

    private val container =
        context.getSystemService(Context.LAYOUT_INFLATER_SERVICE).let { service ->
            val inflater = service as LayoutInflater

            inflater.inflate(R.layout.notification_banner, this)
        }

    private val errorImage = resources.getDrawable(R.drawable.icon_notification_error, null)
    private val warningImage = resources.getDrawable(R.drawable.icon_notification_warning, null)

    private val status: ImageView = container.findViewById(R.id.notification_status)
    private val title: TextView = container.findViewById(R.id.notification_title)
    private val message: TextView = container.findViewById(R.id.notification_message)
    private val icon: View = container.findViewById(R.id.notification_icon)

    val notifications = InAppNotificationController { notification ->
        if (notification != null) {
            update(notification)
        }

        animateChange()
    }

    constructor(context: Context) : super(context) {}
    constructor(context: Context, attributes: AttributeSet) : super(context, attributes) {}

    constructor(context: Context, attributes: AttributeSet, defaultStyleAttribute: Int) :
        super(context, attributes, defaultStyleAttribute) {}

    constructor(
        context: Context,
        attributes: AttributeSet,
        defaultStyleAttribute: Int,
        defaultStyleResource: Int
    ) : super(context, attributes, defaultStyleAttribute, defaultStyleResource) {}

    init {
        setBackgroundResource(R.color.darkBlue)

        setOnClickListener {
            jobTracker.newUiJob("click") { onClick() }
        }
    }

    fun onResume() {
        notifications.onResume()
    }

    fun onPause() {
        notifications.onPause()
    }

    fun onDestroy() {
        notifications.onDestroy()
    }

    private suspend fun onClick() {
        notifications.current?.onClick?.let { action ->
            alpha = 0.5f
            setClickable(false)

            jobTracker.runOnBackground(action)

            setClickable(true)
            alpha = 1.0f
        }
    }

    private fun update(notification: InAppNotification) {
        val notificationMessage = notification.message
        val clickAction = notification.onClick

        when (notification.status) {
            StatusLevel.Error -> status.setImageDrawable(errorImage)
            StatusLevel.Warning -> status.setImageDrawable(warningImage)
        }

        title.text = notification.title

        if (notificationMessage != null) {
            message.text = notificationMessage
            message.visibility = View.VISIBLE
        } else {
            message.visibility = View.GONE
        }

        if (notification.showIcon) {
            icon.visibility = View.VISIBLE
        } else {
            icon.visibility = View.GONE
        }

        setClickable(clickAction != null)
    }

    private fun animateChange() {
        val shouldShow = notifications.current != null

        if (shouldShow && visibility == View.INVISIBLE) {
            visibility = View.VISIBLE
            translationY = -height.toFloat()
            animate().translationY(0.0F).setDuration(350).start()
        } else if (!shouldShow && visibility == View.VISIBLE) {
            animate().translationY(-height.toFloat()).setDuration(350).withEndAction {
                visibility = View.INVISIBLE
            }
        }
    }
}
