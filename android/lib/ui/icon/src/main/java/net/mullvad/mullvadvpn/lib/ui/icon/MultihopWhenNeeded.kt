package net.mullvad.mullvadvpn.lib.ui.icon

import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.graphics.vector.group
import androidx.compose.ui.graphics.vector.path
import androidx.compose.ui.unit.dp

val MultihopWhenNeeded: ImageVector
    get() {
        if (_MultihopWhenNeeded != null) return _MultihopWhenNeeded!!

        _MultihopWhenNeeded =
            ImageVector.Builder(
                    name = "MultihopWhenNeeded",
                    defaultWidth = 24.dp,
                    defaultHeight = 24.dp,
                    viewportWidth = 24f,
                    viewportHeight = 24f,
                )
                .apply {
                    path {}
                    group {
                        path(fill = SolidColor(Color(0xFFFFFFFF))) {
                            moveTo(12f, 21.325f)
                            curveTo(11.7667f, 21.325f, 11.5333f, 21.2833f, 11.3f, 21.2f)
                            curveTo(11.0667f, 21.1167f, 10.8583f, 20.9917f, 10.675f, 20.825f)
                            curveTo(9.59167f, 19.825f, 8.63333f, 18.85f, 7.8f, 17.9f)
                            curveTo(6.96667f, 16.95f, 6.27083f, 16.0292f, 5.7125f, 15.1375f)
                            curveTo(5.15417f, 14.2458f, 4.72917f, 13.3875f, 4.4375f, 12.5625f)
                            curveTo(4.14583f, 11.7375f, 4f, 10.95f, 4f, 10.2f)
                            curveTo(4f, 8.48334f, 4.43333f, 6.94167f, 5.3f, 5.57501f)
                            curveTo(6.16667f, 4.20834f, 7.39167f, 3.20834f, 8.975f, 2.575f)
                            curveTo(9.40833f, 2.39167f, 9.8875f, 2.25f, 10.4125f, 2.15f)
                            curveTo(10.9375f, 2.05f, 11.4583f, 1.99167f, 11.975f, 1.975f)
                            curveTo(12.2583f, 1.975f, 12.4958f, 2.06667f, 12.6875f, 2.25f)
                            curveTo(12.8792f, 2.43334f, 12.975f, 2.66667f, 12.975f, 2.95f)
                            curveTo(12.975f, 3.23334f, 12.8792f, 3.47917f, 12.6875f, 3.6875f)
                            curveTo(12.4958f, 3.89584f, 12.2583f, 4f, 11.975f, 4f)
                            curveTo(11.575f, 4f, 11.1833f, 4.0375f, 10.8f, 4.1125f)
                            curveTo(10.4167f, 4.18751f, 10.0333f, 4.3f, 9.65f, 4.45001f)
                            curveTo(8.48333f, 4.93334f, 7.58333f, 5.7f, 6.95f, 6.75f)
                            curveTo(6.31667f, 7.8f, 6f, 8.95f, 6f, 10.2f)
                            curveTo(6f, 11.3833f, 6.49167f, 12.7375f, 7.475f, 14.2625f)
                            curveTo(8.45833f, 15.7875f, 9.96667f, 17.4833f, 12f, 19.35f)
                            curveTo(13.0167f, 18.4167f, 13.9f, 17.5292f, 14.65f, 16.6875f)
                            curveTo(15.4f, 15.8458f, 16.025f, 15.0417f, 16.525f, 14.275f)
                            curveTo(16.8583f, 13.7583f, 17.15f, 13.2292f, 17.4f, 12.6875f)
                            curveTo(17.65f, 12.1458f, 17.825f, 11.5833f, 17.925f, 11f)
                            curveTo(17.9917f, 10.6167f, 18.1375f, 10.35f, 18.3625f, 10.2f)
                            curveTo(18.5875f, 10.05f, 18.825f, 9.99167f, 19.075f, 10.025f)
                            curveTo(19.325f, 10.0583f, 19.5375f, 10.1583f, 19.7125f, 10.325f)
                            curveTo(19.8875f, 10.4917f, 19.9583f, 10.7167f, 19.925f, 11f)
                            curveTo(19.7917f, 12.0333f, 19.4958f, 13.0042f, 19.0375f, 13.9125f)
                            curveTo(18.5792f, 14.8208f, 18.0417f, 15.6917f, 17.425f, 16.525f)
                            curveTo(16.7083f, 17.475f, 15.9708f, 18.3208f, 15.2125f, 19.0625f)
                            curveTo(14.4542f, 19.8042f, 13.825f, 20.3917f, 13.325f, 20.825f)
                            curveTo(13.1417f, 20.9917f, 12.9333f, 21.1167f, 12.7f, 21.2f)
                            curveTo(12.4667f, 21.2833f, 12.2333f, 21.325f, 12f, 21.325f)
                            close()
                            moveTo(12f, 12f)
                            curveTo(12.55f, 12f, 13.0208f, 11.8042f, 13.4125f, 11.4125f)
                            curveTo(13.8042f, 11.0208f, 14f, 10.55f, 14f, 10f)
                            curveTo(14f, 9.45f, 13.8042f, 8.97917f, 13.4125f, 8.58751f)
                            curveTo(13.0208f, 8.19584f, 12.55f, 8f, 12f, 8f)
                            curveTo(11.45f, 8f, 10.9792f, 8.19584f, 10.5875f, 8.58751f)
                            curveTo(10.1958f, 8.97917f, 10f, 9.45f, 10f, 10f)
                            curveTo(10f, 10.55f, 10.1958f, 11.0208f, 10.5875f, 11.4125f)
                            curveTo(10.9792f, 11.8042f, 11.45f, 12f, 12f, 12f)
                            close()
                        }
                    }
                    group {
                        path(fill = SolidColor(Color(0xFFFFFFFF))) {
                            moveTo(3.77607f, 16.2936f)
                            curveTo(3.86225f, 16.1252f, 4.10734f, 16.1252f, 4.19352f, 16.2936f)
                            lineTo(5.04118f, 17.9494f)
                            curveTo(5.06382f, 17.9935f, 5.10108f, 18.0291f, 5.14623f, 18.0511f)
                            lineTo(6.84066f, 18.8795f)
                            curveTo(7.0125f, 18.9639f, 7.01271f, 19.2041f, 6.84066f, 19.2883f)
                            lineTo(5.14623f, 20.1157f)
                            curveTo(5.10098f, 20.1378f, 5.06381f, 20.1741f, 5.04118f, 20.2184f)
                            lineTo(4.19352f, 21.8741f)
                            curveTo(4.10721f, 22.042f, 3.86237f, 22.042f, 3.77607f, 21.8741f)
                            lineTo(2.92841f, 20.2184f)
                            curveTo(2.90583f, 20.1742f, 2.86938f, 20.1378f, 2.82427f, 20.1157f)
                            lineTo(1.12893f, 19.2883f)
                            curveTo(0.956917f, 19.2041f, 0.957126f, 18.9639f, 1.12893f, 18.8795f)
                            lineTo(2.82427f, 18.0511f)
                            curveTo(2.86925f, 18.0291f, 2.90583f, 17.9933f, 2.92841f, 17.9494f)
                            lineTo(3.77607f, 16.2936f)
                            close()
                            moveTo(18.7988f, 0.756319f)
                            curveTo(18.885f, 0.5879f, 19.1301f, 0.5879f, 19.2162f, 0.756319f)
                            lineTo(20.2201f, 2.71646f)
                            curveTo(20.2427f, 2.76051f, 20.2792f, 2.79612f, 20.3242f, 2.81822f)
                            lineTo(22.331f, 3.79918f)
                            curveTo(22.5029f, 3.88357f, 22.5031f, 4.12379f, 22.331f, 4.20799f)
                            lineTo(20.3242f, 5.18806f)
                            curveTo(20.2792f, 5.21013f, 20.2427f, 5.24579f, 20.2201f, 5.28982f)
                            lineTo(19.2162f, 7.25085f)
                            curveTo(19.1299f, 7.41869f, 18.8851f, 7.41869f, 18.7988f, 7.25085f)
                            lineTo(17.7949f, 5.28982f)
                            curveTo(17.7723f, 5.24579f, 17.7359f, 5.21013f, 17.6908f, 5.18806f)
                            lineTo(15.684f, 4.20799f)
                            curveTo(15.5119f, 4.12379f, 15.5122f, 3.88357f, 15.684f, 3.79918f)
                            lineTo(17.6908f, 2.81822f)
                            curveTo(17.7359f, 2.79613f, 17.7723f, 2.76052f, 17.7949f, 2.71646f)
                            lineTo(18.7988f, 0.756319f)
                            close()
                        }
                    }
                }
                .build()

        return _MultihopWhenNeeded!!
    }

private var _MultihopWhenNeeded: ImageVector? = null
