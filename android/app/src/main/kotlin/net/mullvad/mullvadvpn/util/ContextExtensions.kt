package net.mullvad.mullvadvpn.util

import android.app.Activity
import android.content.Context
import android.content.ContextWrapper
import android.content.Intent
import android.os.Build
import android.os.Bundle
import java.io.Serializable

fun Context.getActivity(): Activity? {
    return when (this) {
        is Activity -> this
        is ContextWrapper -> this.baseContext.getActivity()
        else -> null
    }
}

inline fun <reified T : Serializable> Bundle.serializable(key: String): T? =
    when {
        Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU -> getSerializable(key, T::class.java)
        else -> @Suppress("DEPRECATION") getSerializable(key) as? T
    }

inline fun <reified T : Serializable> Intent.serializable(key: String): T? =
    when {
        Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU ->
            getSerializableExtra(key, T::class.java)

        else -> @Suppress("DEPRECATION") getSerializableExtra(key) as? T
    }
