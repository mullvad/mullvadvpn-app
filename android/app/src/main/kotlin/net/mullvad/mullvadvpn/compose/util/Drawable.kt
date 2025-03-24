package net.mullvad.mullvadvpn.compose.util

import android.graphics.drawable.BitmapDrawable
import android.graphics.drawable.Drawable

private const val MAX_BITMAP_SIZE_BYTES = 100 * 1024 * 1024

fun Drawable.isBelowMaxSize(): Boolean =
    if (this is BitmapDrawable) {
        bitmap.byteCount < MAX_BITMAP_SIZE_BYTES
    } else {
        true
    }
