package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize
import org.joda.time.DateTime

@Parcelize
data class Device(
    val id: DeviceId,
    private val name: String,
    val pubkey: ByteArray,
    val created: DateTime
) : Parcelable {

    fun displayName(): String = name.capitalizeFirstCharOfEachWord()

    override fun equals(other: Any?): Boolean {
        if (this === other) return true
        if (javaClass != other?.javaClass) return false

        other as Device

        if (id != other.id) return false
        if (name != other.name) return false
        if (!pubkey.contentEquals(other.pubkey)) return false
        if (created != other.created) return false

        return true
    }

    override fun hashCode(): Int {
        var result = id.hashCode()
        result = 31 * result + name.hashCode()
        result = 31 * result + pubkey.contentHashCode()
        result = 31 * result + created.hashCode()
        return result
    }
}

private fun String.capitalizeFirstCharOfEachWord(): String {
    return split(" ")
        .joinToString(" ") { word -> word.replaceFirstChar { firstChar -> firstChar.uppercase() } }
        .trimEnd()
}
