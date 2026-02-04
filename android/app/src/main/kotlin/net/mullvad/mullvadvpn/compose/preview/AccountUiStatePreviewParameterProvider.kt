package net.mullvad.mullvadvpn.compose.preview

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import java.time.ZonedDateTime
import java.time.format.DateTimeFormatter
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.common.toLc
import net.mullvad.mullvadvpn.lib.model.AccountNumber
import net.mullvad.mullvadvpn.viewmodel.AccountUiState

class AccountUiStatePreviewParameterProvider : PreviewParameterProvider<Lc<Unit, AccountUiState>> {
    override val values =
        sequenceOf(
            Lc.Loading(Unit),
            AccountUiState(
                    deviceName = "Test Name",
                    accountNumber = AccountNumber("1234123412341234"),
                    accountExpiry =
                        ZonedDateTime.parse(
                            "2050-12-01T00:00:00.000Z",
                            DateTimeFormatter.ISO_ZONED_DATE_TIME,
                        ),
                    showLogoutLoading = false,
                    verificationPending = true,
                )
                .toLc(),
            AccountUiState(
                    deviceName = "Test Name",
                    accountNumber = AccountNumber("1234123412341234"),
                    accountExpiry =
                        ZonedDateTime.parse(
                            "2050-12-01T00:00:00.000Z",
                            DateTimeFormatter.ISO_ZONED_DATE_TIME,
                        ),
                    showLogoutLoading = true,
                    verificationPending = false,
                )
                .toLc(),
        )
}
