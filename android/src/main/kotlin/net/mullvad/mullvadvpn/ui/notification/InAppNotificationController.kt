package net.mullvad.mullvadvpn.ui.notification

import kotlin.properties.Delegates.observable

class InAppNotificationController(private val onNotificationChanged: (InAppNotification?) -> Unit) {
    private val indices = HashMap<InAppNotification, Int>()
    private val notifications = ArrayList<InAppNotification>()

    private var currentIndex: Int? = null

    var current by observable<InAppNotification?>(null) { _, oldNotification, newNotification ->
        if (oldNotification != newNotification) {
            onNotificationChanged.invoke(newNotification)
        }
    }

    fun register(notification: InAppNotification) {
        notification.controller = this

        indices.put(notification, notifications.size)
        notifications.add(notification)

        notificationChanged(notification)
    }

    fun onResume() {
        for (notification in notifications) {
            notification.onResume()
        }
    }

    fun onPause() {
        for (notification in notifications) {
            notification.onPause()
        }
    }

    fun onDestroy() {
        for (notification in notifications) {
            notification.onDestroy()
        }
    }

    fun notificationChanged(notification: InAppNotification) {
        if (notification.shouldShow) {
            maybeShowNotification(notification)
        } else {
            maybeHideNotification(notification)
        }
    }

    private fun maybeShowNotification(notification: InAppNotification) {
        indices.get(notification)?.let { index ->
            if (index <= (currentIndex ?: Int.MAX_VALUE)) {
                current = notification
                currentIndex = index
            }
        }
    }

    private fun maybeHideNotification(notification: InAppNotification) {
        if (current == notification) {
            val start = currentIndex!! + 1
            val end = notifications.size

            for (index in start until end) {
                val candidate = notifications.get(index)

                if (candidate.shouldShow) {
                    current = candidate
                    currentIndex = index
                    return
                }
            }

            current = null
            currentIndex = null
        }
    }
}
