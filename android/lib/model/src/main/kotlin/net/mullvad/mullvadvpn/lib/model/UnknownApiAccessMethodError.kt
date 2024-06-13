package net.mullvad.mullvadvpn.lib.model

data class UnknownApiAccessMethodError(val throwable: Throwable) : UpdateApiAccessMethodError
