package net.mullvad.mullvadvpn.lib.model

import androidx.datastore.core.CorruptionException
import androidx.datastore.core.Serializer
import com.google.protobuf.InvalidProtocolBufferException
import java.io.InputStream
import java.io.OutputStream
import net.mullvad.mullvadvpn.model.TunnelPreference

object TunnelPreferencesSerializer : Serializer<TunnelPreference> {
    override val defaultValue: TunnelPreference = TunnelPreference.getDefaultInstance()

    override suspend fun readFrom(input: InputStream): TunnelPreference {
        try {
            return TunnelPreference.parseFrom(input)
        } catch (exception: InvalidProtocolBufferException) {
            throw CorruptionException("Cannot read proto", exception)
        }
    }

    override suspend fun writeTo(t: TunnelPreference, output: OutputStream) = t.writeTo(output)
}
