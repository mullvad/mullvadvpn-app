package net.mullvad.mullvadvpn.util

import android.content.res.Resources

data class ErrorNotificationMessage(
    val titleResourceId: Int,
    val messageResourceId: Int,
    val optionalMessageArgument: String? = null
) {
    fun getTitleText(resources: Resources): String {
        return resources.getString(titleResourceId)
    }

    fun getMessageText(resources: Resources): String {
        return if (optionalMessageArgument != null) {
            resources.getString(messageResourceId, optionalMessageArgument)
        } else {
            resources.getString(messageResourceId)
        }
    }
}
