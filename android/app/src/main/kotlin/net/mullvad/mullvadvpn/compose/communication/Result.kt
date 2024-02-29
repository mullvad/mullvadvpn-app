package net.mullvad.mullvadvpn.compose.communication

data class Result<T : CustomListAction>(val reverseAction: T, val messageParams: List<String>)
