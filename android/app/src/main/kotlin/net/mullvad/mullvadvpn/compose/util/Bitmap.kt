package net.mullvad.mullvadvpn.compose.util

import android.graphics.Bitmap

private const val MAX_BITMAP_SIZE_BYTES = 100 * 1024 * 1024

fun Bitmap.isBelowMaxBitmapSize(): Boolean = byteCount < MAX_BITMAP_SIZE_BYTES
