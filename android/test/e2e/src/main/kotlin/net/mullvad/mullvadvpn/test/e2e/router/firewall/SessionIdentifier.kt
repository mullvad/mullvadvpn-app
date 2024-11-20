package net.mullvad.mullvadvpn.test.e2e.router.firewall

import android.annotation.SuppressLint
import android.provider.Settings
import androidx.test.platform.app.InstrumentationRegistry
import java.util.UUID
import kotlinx.serialization.Serializable

@JvmInline
@Serializable
value class SessionIdentifier(val value: String) {
    override fun toString(): String = value

    companion object {
        @SuppressLint("HardwareIds")
        fun fromDeviceIdentifier(): SessionIdentifier {
            val deviceIdentifier =
                Settings.Secure.getString(
                    InstrumentationRegistry.getInstrumentation().targetContext.contentResolver,
                    Settings.Secure.ANDROID_ID,
                )

            return SessionIdentifier(UUID.nameUUIDFromBytes(deviceIdentifier.toByteArray()).toString())
        }
    }
}
