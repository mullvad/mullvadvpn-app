package net.mullvad.mullvadvpn.ui.notification

import java.util.PriorityQueue
import kotlin.properties.Delegates.observable

class InAppNotificationController(private val onNotificationChanged: (InAppNotification?) -> Unit) {
    private val notificationPrioritizer = object : Comparator<InAppNotification> {
        override fun compare(left: InAppNotification, right: InAppNotification) =
            if (left.shouldShow != right.shouldShow) {
                if (left.shouldShow) { -1 } else { 1 }
            } else if (left.status != right.status) {
                left.status.compareTo(right.status)
            } else if (left != right) {
                indices.get(left)!!.compareTo(indices.get(right)!!)
            } else {
                0
            }
    }

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
