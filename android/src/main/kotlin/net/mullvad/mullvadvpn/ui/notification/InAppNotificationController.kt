package net.mullvad.mullvadvpn.ui.notification

import java.util.PriorityQueue
import kotlin.properties.Delegates.observable

class InAppNotificationController(private val onNotificationChanged: (InAppNotification?) -> Unit) {
    private val notificationPrioritizer =
        compareByDescending<InAppNotification> { it.shouldShow }
            .thenBy { it.status }
            .thenBy { indices.get(it)!! }

    private val activeNotifications = PriorityQueue(notificationPrioritizer)
    private val indices = HashMap<InAppNotification, Int>()
    private val notifications = ArrayList<InAppNotification>()

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
        if (notification.shouldShow && !activeNotifications.contains(notification)) {
            activeNotifications.add(notification)
        } else {
            activeNotifications.remove(notification)
        }

        current = activeNotifications.peek()
    }
}
