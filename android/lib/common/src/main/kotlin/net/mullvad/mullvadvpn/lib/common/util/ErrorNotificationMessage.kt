package net.mullvad.mullvadvpn.lib.common.util

data class ErrorNotificationMessage(
    val titleResourceId: Int,
    val messageResourceId: Int,
    val optionalMessageArgument: String? = null
)
