package net.mullvad.mullvadvpn.data

import io.mockk.mockk
import java.time.ZonedDateTime
import net.mullvad.mullvadvpn.lib.model.AccountData

fun AccountData.Companion.mock(expiry: ZonedDateTime): AccountData =
    AccountData(
        id = mockk(relaxed = true),
        accountNumber = mockk(relaxed = true),
        expiryDate = expiry,
    )
