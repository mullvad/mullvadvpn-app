package net.mullvad.mullvadvpn.feature.home.impl.data

import io.mockk.mockk
import net.mullvad.mullvadvpn.lib.model.AccountData

fun AccountData.Companion.mock(hasPayments: Boolean): AccountData =
    AccountData(
        id = mockk(relaxed = true),
        accountNumber = mockk(relaxed = true),
        expiryDate = mockk(relaxed = true),
        hasPayments = hasPayments,
    )
