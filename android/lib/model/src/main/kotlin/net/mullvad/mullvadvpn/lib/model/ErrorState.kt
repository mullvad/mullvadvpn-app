package net.mullvad.mullvadvpn.lib.model

data class ErrorState(val cause: ErrorStateCause, val isBlocking: Boolean)
