package net.mullvad.mullvadvpn.test.api.constant

import android.os.Bundle
import androidx.test.platform.app.InstrumentationRegistry
import net.mullvad.mullvadvpn.test.common.extension.getRequiredArgument

fun Bundle.getPartnerAuth(infrastructure: String) =
    InstrumentationRegistry.getArguments().getString("mullvad.test.e2e.$infrastructure.partnerAuth")

fun Bundle.getValidAccountNumber(infrastructure: String) =
    InstrumentationRegistry.getArguments()
        .getRequiredArgument("mullvad.test.e2e.$infrastructure.accountNumber.valid")

fun Bundle.getInvalidAccountNumber(infrastructure: String) =
    InstrumentationRegistry.getArguments()
        .getRequiredArgument("mullvad.test.e2e.$infrastructure.accountNumber.invalid")
