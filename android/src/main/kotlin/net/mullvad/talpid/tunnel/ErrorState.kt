package net.mullvad.talpid.tunnel

data class ErrorState(val cause: BlockReason, val isBlocking: Boolean)
