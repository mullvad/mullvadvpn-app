package net.mullvad.mullvadvpn.lib.ui.icon

import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.graphics.vector.group
import androidx.compose.ui.graphics.vector.path
import androidx.compose.ui.unit.dp

val LocationOn: ImageVector
    get() {
        if (_LocationOn != null) return _LocationOn!!

        _LocationOn = ImageVector.Builder(
            name = "LocationOn",
            defaultWidth = 18.dp,
            defaultHeight = 18.dp,
            viewportWidth = 18f,
            viewportHeight = 18f
        ).apply {
            path {
            }
            group {
                path(
                    fill = SolidColor(Color(0xFFFFFFFF))
                ) {
                    moveTo(9f, 14.5125f)
                    curveTo(10.525f, 13.1125f, 11.6563f, 11.8406f, 12.3938f, 10.6969f)
                    curveTo(13.1313f, 9.55312f, 13.5f, 8.5375f, 13.5f, 7.65f)
                    curveTo(13.5f, 6.2875f, 13.0656f, 5.17187f, 12.1969f, 4.30312f)
                    curveTo(11.3281f, 3.43437f, 10.2625f, 3f, 9f, 3f)
                    curveTo(7.7375f, 3f, 6.67187f, 3.43437f, 5.80312f, 4.30312f)
                    curveTo(4.93437f, 5.17187f, 4.5f, 6.2875f, 4.5f, 7.65f)
                    curveTo(4.5f, 8.5375f, 4.86875f, 9.55312f, 5.60625f, 10.6969f)
                    curveTo(6.34375f, 11.8406f, 7.475f, 13.1125f, 9f, 14.5125f)
                    close()
                    moveTo(9f, 15.9938f)
                    curveTo(8.825f, 15.9938f, 8.65f, 15.9625f, 8.475f, 15.9f)
                    curveTo(8.3f, 15.8375f, 8.14375f, 15.7438f, 8.00625f, 15.6188f)
                    curveTo(7.19375f, 14.8688f, 6.475f, 14.1375f, 5.85f, 13.425f)
                    curveTo(5.225f, 12.7125f, 4.70312f, 12.0219f, 4.28437f, 11.3531f)
                    curveTo(3.86562f, 10.6844f, 3.54688f, 10.0406f, 3.32812f, 9.42188f)
                    curveTo(3.10937f, 8.80313f, 3f, 8.2125f, 3f, 7.65f)
                    curveTo(3f, 5.775f, 3.60313f, 4.28125f, 4.80938f, 3.16875f)
                    curveTo(6.01563f, 2.05625f, 7.4125f, 1.5f, 9f, 1.5f)
                    curveTo(10.5875f, 1.5f, 11.9844f, 2.05625f, 13.1906f, 3.16875f)
                    curveTo(14.3969f, 4.28125f, 15f, 5.775f, 15f, 7.65f)
                    curveTo(15f, 8.2125f, 14.8906f, 8.80313f, 14.6719f, 9.42188f)
                    curveTo(14.4531f, 10.0406f, 14.1344f, 10.6844f, 13.7156f, 11.3531f)
                    curveTo(13.2969f, 12.0219f, 12.775f, 12.7125f, 12.15f, 13.425f)
                    curveTo(11.525f, 14.1375f, 10.8062f, 14.8688f, 9.99375f, 15.6188f)
                    curveTo(9.85625f, 15.7438f, 9.7f, 15.8375f, 9.525f, 15.9f)
                    curveTo(9.35f, 15.9625f, 9.175f, 15.9938f, 9f, 15.9938f)
                    close()
                    moveTo(9f, 9f)
                    curveTo(9.4125f, 9f, 9.76563f, 8.85313f, 10.0594f, 8.55938f)
                    curveTo(10.3531f, 8.26563f, 10.5f, 7.9125f, 10.5f, 7.5f)
                    curveTo(10.5f, 7.0875f, 10.3531f, 6.73438f, 10.0594f, 6.44063f)
                    curveTo(9.76563f, 6.14688f, 9.4125f, 6f, 9f, 6f)
                    curveTo(8.5875f, 6f, 8.23438f, 6.14688f, 7.94063f, 6.44063f)
                    curveTo(7.64688f, 6.73438f, 7.5f, 7.0875f, 7.5f, 7.5f)
                    curveTo(7.5f, 7.9125f, 7.64688f, 8.26563f, 7.94063f, 8.55938f)
                    curveTo(8.23438f, 8.85313f, 8.5875f, 9f, 9f, 9f)
                    close()
                }
            }
        }.build()

        return _LocationOn!!
    }

private var _LocationOn: ImageVector? = null
