package net.mullvad.mullvadvpn

class MullvadDaemon {
    init {
        System.loadLibrary("mullvad_jni")
        initialize()
    }

    private external fun initialize()
}
