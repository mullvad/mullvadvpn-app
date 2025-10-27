package net.mullvad.mullvadvpn.lib.repository

import android.content.Context
import arrow.core.Either
import net.mullvad.mullvadvpn.lib.common.util.prepareVpnSafe
import net.mullvad.mullvadvpn.lib.model.PrepareError
import net.mullvad.mullvadvpn.lib.model.Prepared

class PrepareVpnUseCase(private val applicationContext: Context) {
    fun invoke(): Either<PrepareError, Prepared> = applicationContext.prepareVpnSafe()
}
