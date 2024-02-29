package net.mullvad.mullvadvpn.compose.communication

@JvmInline value class Request<T : CustomListAction>(val action: T)
