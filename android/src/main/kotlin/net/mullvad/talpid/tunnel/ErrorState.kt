package net.mullvad.talpid.tunnel

data class ErrorState(val cause: ErrorStateCause, val isBlocking: Boolean)
