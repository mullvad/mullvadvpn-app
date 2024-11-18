package net.mullvad.mullvadvpn.lib.shared

import android.content.Context
import arrow.core.Either
import net.mullvad.mullvadvpn.lib.common.util.prepareVpnSafe
import net.mullvad.mullvadvpn.lib.model.PrepareError
import net.mullvad.mullvadvpn.lib.model.Prepared

class VpnProfileUseCase(private val applicationContext: Context) {
    fun prepareVpn(): Either<PrepareError, Prepared> = applicationContext.prepareVpnSafe()
}
