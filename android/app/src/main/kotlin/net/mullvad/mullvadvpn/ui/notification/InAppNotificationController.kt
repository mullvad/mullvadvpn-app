package net.mullvad.mullvadvpn.ui.notification

import java.util.PriorityQueue
import kotlin.properties.Delegates.observable

class InAppNotificationController(private val onNotificationChanged: (InAppNotification?) -> Unit) {
    private val notificationPrioritizer =
        compareByDescending<InAppNotification> { it.shouldShow }
            .thenBy { it.status }
            .thenBy { notifications.get(it)!! }

    private val activeNotifications = PriorityQueue(notificationPrioritizer)
    private val notifications = HashMap<InAppNotification, Int>()

    var current by observable<InAppNotification?>(null) { _, oldNotification, newNotification ->
        if (oldNotification != newNotification) {
            onNotificationChanged.invoke(newNotification)
        }
    }

    fun register(notification: InAppNotification) {
        notification.controller = this

        notifications.put(notification, notifications.size)

        notificationChanged(notification)
    }

    fun onResume() {
        for (notification in notifications.keys) {
            notification.onResume()
        }
    }

    fun onPause() {
        for (notification in notifications.keys) {
            notification.onPause()
        }
    }

    fun onDestroy() {
        for (notification in notifications.keys) {
            notification.onDestroy()
        }
    }

    fun notificationChanged(notification: InAppNotification) {
        if (notification.shouldShow && !activeNotifications.contains(notification)) {
            activeNotifications.add(notification)
        } else {
            if (!notification.shouldShow) activeNotifications.remove(notification)
        }

        current = activeNotifications.peek()
    }
}
