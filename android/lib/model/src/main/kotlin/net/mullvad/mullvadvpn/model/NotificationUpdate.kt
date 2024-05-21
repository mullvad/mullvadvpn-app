package net.mullvad.mullvadvpn.model

sealed interface NotificationUpdate<out D> {
    val notificationId: NotificationId

    data class Notify<D>(override val notificationId: NotificationId, val value: D) :
        NotificationUpdate<D>

    data class Cancel(override val notificationId: NotificationId) : NotificationUpdate<Nothing>
}
