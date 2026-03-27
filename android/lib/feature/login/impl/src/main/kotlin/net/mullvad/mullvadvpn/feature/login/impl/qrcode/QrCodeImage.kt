package net.mullvad.mullvadvpn.feature.login.impl.qrcode

import android.graphics.Bitmap
import androidx.compose.foundation.Image
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.asImageBitmap
import com.google.zxing.BarcodeFormat
import com.google.zxing.EncodeHintType
import com.google.zxing.qrcode.QRCodeWriter
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import androidx.core.graphics.createBitmap

@Composable
fun QRCodeImage(
    modifier: Modifier = Modifier,
    text: String,
    sizePx: Int = 512 // Default to a high-res 512x512 image
) {
    // Hold the generated bitmap in state
    var bitmap by remember { mutableStateOf<Bitmap?>(null) }

    // Re-generate the QR code if the text or size changes
    LaunchedEffect(text, sizePx) {
        if (text.isNotEmpty()) {
            bitmap = withContext(Dispatchers.Default) {
                generateQrBitmap(text, sizePx)
            }
        } else {
            bitmap = null
        }
    }

    // Display the generated image
    bitmap?.let { b ->
        Image(
            modifier = modifier,
            bitmap = b.asImageBitmap(),
            contentDescription = "QR Code for $text",
        )
    }
}

// Background generator function
private fun generateQrBitmap(text: String, sizePx: Int): Bitmap {
    // Add a small blank margin around the QR code (1 block)
    val hints = mapOf(EncodeHintType.MARGIN to 1)

    val bitMatrix = QRCodeWriter().encode(text, BarcodeFormat.QR_CODE, sizePx, sizePx, hints)
    val width = bitMatrix.width
    val height = bitMatrix.height

    // Allocate a pixel array for fast bitmap creation
    val pixels = IntArray(width * height)

    for (y in 0 until height) {
        val offset = y * width
        for (x in 0 until width) {
            // Check if the current block is "true" (black) or "false" (white)
            pixels[offset + x] = if (bitMatrix.get(x, y)) {
                android.graphics.Color.BLACK
            } else {
                android.graphics.Color.WHITE // Transparent or WHITE
            }
        }
    }

    val bitmap = createBitmap(width, height)
    bitmap.setPixels(pixels, 0, width, 0, 0, width, height)
    return bitmap
}
