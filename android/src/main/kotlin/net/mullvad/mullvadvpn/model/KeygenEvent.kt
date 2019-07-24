package net.mullvad.mullvadvpn.model

sealed class KeygenEvent {
    class NewKey(var publicKey: PublicKey) : KeygenEvent()
    class TooManyKeys : KeygenEvent()
    class GenerationFailure : KeygenEvent()
}
