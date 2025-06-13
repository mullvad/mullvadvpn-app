package net.mullvad.mullvadvpn.lib.theme

import androidx.compose.material3.Typography
import androidx.compose.ui.text.font.FontWeight
import net.mullvad.mullvadvpn.lib.theme.typeface.TypeScale

/*
The app currently uses the following text styles directly in the code:
headlineLarge (30sp 700 weight) -> Used for title in PrivacyDisclaimer, Welcome and Login
headlineSmall (24sp 600 weight) -> Used for title in OutOfTime, DeviceRevoked, ReportAProblem etc
titleLarge (22sp 600 weight) -> Used for Connection status and location
titleMedium (16sp 600 weight) -> Used for cell header text and button text
bodyLarge (16sp 400 weight) -> Used for title in two row cells and some other non-standard cells
bodyMedium (14sp 400 weight) -> Used for descriptions in screens and descriptions for cells
bodySmall (12sp 400 weight) -> Disclaimer texts and error texts under inputs
labelLarge (14sp 500 weight) -> Cell that are not header cells, Dialog texts, device name and expiry
 */

internal val MullvadMaterial3Typography =
    Typography(
        headlineLarge =
            Typography()
                .headlineLarge
                .copy(fontSize = TypeScale.TextHuge, fontWeight = FontWeight.Bold),
        headlineSmall = Typography().headlineSmall.copy(fontWeight = FontWeight.SemiBold),
        titleLarge = Typography().titleLarge.copy(fontWeight = FontWeight.SemiBold),
        titleMedium = Typography().titleMedium.copy(fontWeight = FontWeight.SemiBold),
    )
