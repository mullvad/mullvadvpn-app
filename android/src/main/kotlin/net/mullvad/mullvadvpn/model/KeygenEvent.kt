package net.mullvad.mullvadvpn.model

sealed class KeygenEvent {
    class NewKey(val publicKey: PublicKey, val verified: Boolean?) : KeygenEvent()
    class TooManyKeys : KeygenEvent()
    class GenerationFailure : KeygenEvent()
}
