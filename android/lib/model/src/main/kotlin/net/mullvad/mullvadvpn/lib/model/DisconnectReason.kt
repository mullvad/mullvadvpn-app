package net.mullvad.mullvadvpn.lib.model

enum class DisconnectReason(val logString: String) {
    USER_INITIATED_DISCONNECT_BUTTON("user-initiated-main-button"),
    USER_INITIATED_CANCEL_BUTTON("user-initiated-cancel-button"),
    USER_INITIATED_NOTIFICATION_TILE("user-initiated-notification-tile"),
    USER_INITIATED_GO_TO_LOGIN("user-initiated-go-to-login"),
    USER_INITIATED_OUT_OF_TIME("user-initiated-out-of-time"),
    USER_INITIATED_WELCOME("user-initiated-welcome"),
    SYSTEM_REVOKE("system-revoke"),
}
