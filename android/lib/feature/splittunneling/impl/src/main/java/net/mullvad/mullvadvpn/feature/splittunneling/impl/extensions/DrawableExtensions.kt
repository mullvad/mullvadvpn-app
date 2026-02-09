package net.mullvad.mullvadvpn.feature.splittunneling.impl.extensions

import android.graphics.drawable.BitmapDrawable
import android.graphics.drawable.Drawable

private const val MAX_BITMAP_SIZE_BYTES = 100 * 1024 * 1024 // 100MB

internal fun Drawable.isBelowMaxByteSize(): Boolean =
    if (this is BitmapDrawable) bitmap.byteCount < MAX_BITMAP_SIZE_BYTES else true

internal fun Drawable.hasValidSize(): Boolean = intrinsicHeight > 0 && intrinsicWidth > 0
