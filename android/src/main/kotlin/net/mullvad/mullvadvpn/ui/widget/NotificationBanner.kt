package net.mullvad.mullvadvpn.ui.widget

import android.animation.Animator
import android.animation.Animator.AnimatorListener
import android.animation.ObjectAnimator
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

    private val animationListener = object : AnimatorListener {
        override fun onAnimationCancel(animation: Animator) {}
        override fun onAnimationRepeat(animation: Animator) {}

        override fun onAnimationStart(animation: Animator) {
            visibility = View.VISIBLE
        }

        override fun onAnimationEnd(animation: Animator) {
            if (reversedAnimation) {
                // Banner is now hidden
                visibility = View.INVISIBLE
            }
        }
    }

    private val animation = ObjectAnimator.ofFloat(this, "translationY", 0.0f).apply {
        addListener(animationListener)
        setDuration(350)
    }

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

    private var reversedAnimation = false

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

    protected override fun onSizeChanged(width: Int, height: Int, oldWidth: Int, oldHeight: Int) {
        animation.setFloatValues(-height.toFloat(), 0.0f)
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
        val notification = notifications.current

        if (notification != null && visibility == View.INVISIBLE) {
            reversedAnimation = false
            update(notification)
            animation.start()
        } else if (!shouldShow && visibility == View.VISIBLE) {
            reversedAnimation = true
            animation.reverse()
        }
    }
}
