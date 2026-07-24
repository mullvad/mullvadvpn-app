package net.mullvad.mullvadvpn.lib.ui.icon

import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.graphics.vector.PathData
import androidx.compose.ui.graphics.vector.group
import androidx.compose.ui.graphics.vector.path
import androidx.compose.ui.unit.dp

val FilterActive: ImageVector
    get() {
        if (_FilterActive != null) {
            return _FilterActive!!
        }
        _FilterActive =
            ImageVector.Builder(
                    name = "StateActive",
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
                            moveTo(13f, 16f)
                            curveTo(13.283f, 16f, 13.521f, 16.095f, 13.713f, 16.287f)
                            curveTo(13.905f, 16.479f, 14f, 16.717f, 14f, 17f)
                            curveTo(14f, 17.283f, 13.905f, 17.521f, 13.713f, 17.713f)
                            curveTo(13.521f, 17.905f, 13.283f, 18f, 13f, 18f)
                            horizontalLineTo(11f)
                            curveTo(10.717f, 18f, 10.479f, 17.905f, 10.287f, 17.713f)
                            curveTo(10.095f, 17.521f, 10f, 17.283f, 10f, 17f)
                            curveTo(10f, 16.717f, 10.095f, 16.479f, 10.287f, 16.287f)
                            curveTo(10.479f, 16.095f, 10.717f, 16f, 11f, 16f)
                            horizontalLineTo(13f)
                            close()
                            moveTo(15.684f, 11f)
                            curveTo(16.368f, 11.455f, 17.152f, 11.772f, 17.995f, 11.914f)
                            curveTo(17.997f, 11.942f, 18f, 11.971f, 18f, 12f)
                            curveTo(18f, 12.283f, 17.905f, 12.521f, 17.713f, 12.713f)
                            curveTo(17.521f, 12.905f, 17.283f, 13f, 17f, 13f)
                            horizontalLineTo(7f)
                            curveTo(6.717f, 13f, 6.479f, 12.905f, 6.287f, 12.713f)
                            curveTo(6.095f, 12.521f, 6f, 12.283f, 6f, 12f)
                            curveTo(6f, 11.717f, 6.095f, 11.479f, 6.287f, 11.287f)
                            curveTo(6.479f, 11.095f, 6.717f, 11f, 7f, 11f)
                            horizontalLineTo(15.684f)
                            close()
                            moveTo(13f, 6f)
                            curveTo(13f, 6.701f, 13.122f, 7.374f, 13.343f, 8f)
                            horizontalLineTo(4f)
                            curveTo(3.717f, 8f, 3.479f, 7.905f, 3.287f, 7.713f)
                            curveTo(3.095f, 7.521f, 3f, 7.283f, 3f, 7f)
                            curveTo(3f, 6.717f, 3.095f, 6.479f, 3.287f, 6.287f)
                            curveTo(3.479f, 6.095f, 3.717f, 6f, 4f, 6f)
                            horizontalLineTo(13f)
                            close()
                        }
                        path(fill = SolidColor(Color.White)) {
                            moveTo(22f, 6f)
                            curveTo(22f, 7.657f, 20.657f, 9f, 19f, 9f)
                            curveTo(17.343f, 9f, 16f, 7.657f, 16f, 6f)
                            curveTo(16f, 4.343f, 17.343f, 3f, 19f, 3f)
                            curveTo(20.657f, 3f, 22f, 4.343f, 22f, 6f)
                            close()
                        }
                    }
                }
                .build()

        return _FilterActive!!
    }

@Suppress("ObjectPropertyName") private var _FilterActive: ImageVector? = null
