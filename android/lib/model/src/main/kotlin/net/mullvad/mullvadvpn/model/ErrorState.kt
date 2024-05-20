package net.mullvad.mullvadvpn.model

data class ErrorState(val cause: ErrorStateCause, val isBlocking: Boolean)
