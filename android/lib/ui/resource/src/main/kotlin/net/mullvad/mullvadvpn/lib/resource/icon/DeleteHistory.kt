package net.mullvad.mullvadvpn.lib.resource.icon

import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.graphics.vector.PathData
import androidx.compose.ui.graphics.vector.group
import androidx.compose.ui.graphics.vector.path
import androidx.compose.ui.unit.dp

val DeleteHistory: ImageVector
    get() {
        if (_DeleteHistory != null) {
            return _DeleteHistory!!
        }
        _DeleteHistory =
            ImageVector.Builder(
                    name = "DeleteHistory",
                    defaultWidth = 24.dp,
                    defaultHeight = 24.dp,
                    viewportWidth = 24f,
                    viewportHeight = 24f,
                )
                .apply {
                    group(
                        clipPathData =
                            PathData {
                                moveTo(0f, 0f)
                                horizontalLineToRelative(24f)
                                verticalLineToRelative(24f)
                                horizontalLineToRelative(-24f)
                                close()
                            }
                    ) {
                        path(fill = SolidColor(Color.White)) {
                            moveTo(13f, 19.95f)
                            curveTo(13f, 20.233f, 12.9f, 20.479f, 12.7f, 20.688f)
                            curveTo(12.5f, 20.896f, 12.258f, 21f, 11.975f, 21f)
                            curveTo(10.958f, 20.983f, 9.992f, 20.821f, 9.075f, 20.513f)
                            curveTo(8.158f, 20.204f, 7.283f, 19.725f, 6.45f, 19.075f)
                            curveTo(5.7f, 18.492f, 5.058f, 17.808f, 4.525f, 17.025f)
                            curveTo(3.992f, 16.242f, 3.592f, 15.375f, 3.325f, 14.425f)
                            curveTo(3.242f, 14.158f, 3.275f, 13.908f, 3.425f, 13.675f)
                            curveTo(3.575f, 13.442f, 3.783f, 13.283f, 4.05f, 13.2f)
                            curveTo(4.317f, 13.117f, 4.567f, 13.146f, 4.8f, 13.288f)
                            curveTo(5.033f, 13.429f, 5.192f, 13.633f, 5.275f, 13.9f)
                            curveTo(5.475f, 14.65f, 5.792f, 15.337f, 6.225f, 15.962f)
                            curveTo(6.658f, 16.587f, 7.167f, 17.125f, 7.75f, 17.575f)
                            curveTo(8.35f, 18.042f, 9.017f, 18.396f, 9.75f, 18.638f)
                            curveTo(10.483f, 18.879f, 11.225f, 19f, 11.975f, 19f)
                            curveTo(12.258f, 19f, 12.5f, 19.092f, 12.7f, 19.275f)
                            curveTo(12.9f, 19.458f, 13f, 19.683f, 13f, 19.95f)
                            close()
                            moveTo(18.5f, 18.925f)
                            lineTo(17.1f, 20.3f)
                            curveTo(16.917f, 20.483f, 16.688f, 20.579f, 16.413f, 20.587f)
                            curveTo(16.138f, 20.596f, 15.9f, 20.5f, 15.7f, 20.3f)
                            curveTo(15.517f, 20.117f, 15.425f, 19.883f, 15.425f, 19.6f)
                            curveTo(15.425f, 19.317f, 15.517f, 19.083f, 15.7f, 18.9f)
                            lineTo(17.1f, 17.5f)
                            lineTo(15.7f, 16.1f)
                            curveTo(15.517f, 15.917f, 15.425f, 15.683f, 15.425f, 15.4f)
                            curveTo(15.425f, 15.117f, 15.517f, 14.883f, 15.7f, 14.7f)
                            curveTo(15.883f, 14.517f, 16.117f, 14.425f, 16.4f, 14.425f)
                            curveTo(16.683f, 14.425f, 16.917f, 14.517f, 17.1f, 14.7f)
                            lineTo(18.5f, 16.1f)
                            lineTo(19.9f, 14.7f)
                            curveTo(20.083f, 14.517f, 20.313f, 14.421f, 20.587f, 14.413f)
                            curveTo(20.862f, 14.404f, 21.1f, 14.5f, 21.3f, 14.7f)
                            curveTo(21.483f, 14.883f, 21.575f, 15.117f, 21.575f, 15.4f)
                            curveTo(21.575f, 15.683f, 21.483f, 15.917f, 21.3f, 16.1f)
                            lineTo(19.925f, 17.5f)
                            lineTo(21.3f, 18.9f)
                            curveTo(21.483f, 19.083f, 21.579f, 19.313f, 21.587f, 19.587f)
                            curveTo(21.596f, 19.862f, 21.5f, 20.1f, 21.3f, 20.3f)
                            curveTo(21.117f, 20.483f, 20.883f, 20.575f, 20.6f, 20.575f)
                            curveTo(20.317f, 20.575f, 20.083f, 20.483f, 19.9f, 20.3f)
                            lineTo(18.5f, 18.925f)
                            close()
                            moveTo(12f, 5f)
                            curveTo(10.85f, 5f, 9.775f, 5.267f, 8.775f, 5.8f)
                            curveTo(7.775f, 6.333f, 6.933f, 7.067f, 6.25f, 8f)
                            horizontalLineTo(8f)
                            curveTo(8.283f, 8f, 8.521f, 8.096f, 8.712f, 8.288f)
                            curveTo(8.904f, 8.479f, 9f, 8.717f, 9f, 9f)
                            curveTo(9f, 9.283f, 8.904f, 9.521f, 8.712f, 9.712f)
                            curveTo(8.521f, 9.904f, 8.283f, 10f, 8f, 10f)
                            horizontalLineTo(4f)
                            curveTo(3.717f, 10f, 3.479f, 9.904f, 3.287f, 9.712f)
                            curveTo(3.096f, 9.521f, 3f, 9.283f, 3f, 9f)
                            verticalLineTo(5f)
                            curveTo(3f, 4.717f, 3.096f, 4.479f, 3.287f, 4.287f)
                            curveTo(3.479f, 4.096f, 3.717f, 4f, 4f, 4f)
                            curveTo(4.283f, 4f, 4.521f, 4.096f, 4.713f, 4.287f)
                            curveTo(4.904f, 4.479f, 5f, 4.717f, 5f, 5f)
                            verticalLineTo(6.35f)
                            curveTo(5.85f, 5.283f, 6.887f, 4.458f, 8.113f, 3.875f)
                            curveTo(9.337f, 3.292f, 10.633f, 3f, 12f, 3f)
                            curveTo(13.25f, 3f, 14.421f, 3.233f, 15.512f, 3.7f)
                            curveTo(16.604f, 4.167f, 17.558f, 4.808f, 18.375f, 5.625f)
                            curveTo(19.042f, 6.292f, 19.6f, 7.058f, 20.05f, 7.925f)
                            curveTo(20.5f, 8.792f, 20.792f, 9.717f, 20.925f, 10.7f)
                            curveTo(20.958f, 10.983f, 20.896f, 11.238f, 20.737f, 11.462f)
                            curveTo(20.579f, 11.688f, 20.358f, 11.817f, 20.075f, 11.85f)
                            curveTo(19.792f, 11.883f, 19.538f, 11.821f, 19.313f, 11.663f)
                            curveTo(19.087f, 11.504f, 18.958f, 11.283f, 18.925f, 11f)
                            curveTo(18.808f, 10.233f, 18.583f, 9.517f, 18.25f, 8.85f)
                            curveTo(17.917f, 8.183f, 17.483f, 7.583f, 16.95f, 7.05f)
                            curveTo(16.317f, 6.417f, 15.579f, 5.917f, 14.738f, 5.55f)
                            curveTo(13.896f, 5.183f, 12.983f, 5f, 12f, 5f)
                            close()
                            moveTo(13f, 11.6f)
                            lineTo(14.2f, 12.8f)
                            curveTo(14.433f, 13.033f, 14.538f, 13.279f, 14.512f, 13.538f)
                            curveTo(14.488f, 13.796f, 14.383f, 14.017f, 14.2f, 14.2f)
                            curveTo(14.017f, 14.383f, 13.796f, 14.488f, 13.538f, 14.512f)
                            curveTo(13.279f, 14.538f, 13.033f, 14.433f, 12.8f, 14.2f)
                            lineTo(11.3f, 12.7f)
                            curveTo(11.2f, 12.6f, 11.125f, 12.488f, 11.075f, 12.363f)
                            curveTo(11.025f, 12.238f, 11f, 12.108f, 11f, 11.975f)
                            verticalLineTo(8f)
                            curveTo(11f, 7.717f, 11.096f, 7.479f, 11.288f, 7.287f)
                            curveTo(11.479f, 7.096f, 11.717f, 7f, 12f, 7f)
                            curveTo(12.283f, 7f, 12.521f, 7.096f, 12.712f, 7.287f)
                            curveTo(12.904f, 7.479f, 13f, 7.717f, 13f, 8f)
                            verticalLineTo(11.6f)
                            close()
                        }
                    }
                }
                .build()

        return _DeleteHistory!!
    }

@Suppress("ObjectPropertyName") private var _DeleteHistory: ImageVector? = null
