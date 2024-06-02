package net.mullvad.mullvadvpn.lib.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

@Parcelize
@JvmInline
value class Cipher private constructor(val value: String) : Parcelable {

    override fun toString(): String = value

    companion object {
        fun fromString(cipher: String) = Cipher(Ciphers.fromString(cipher).label)

        fun listAll() = Ciphers.entries.map { Cipher(it.label) }.sortedBy { it.value }

        fun first() = listAll().first()
    }
}

// All suppported shadowsocks ciphers
private enum class Ciphers(val label: String) {
    AES_128_CFB("aes-128-cfb"),
    AES_128_CFB1("aes-128-cfb1"),
    AES_128_CFB8("aes-128-cfb8"),
    AES_128_CFB128("aes-128-cfb128"),
    AES_256_CFB("aes-256-cfb"),
    AES_256_CFB1("aes-256-cfb1"),
    AES_256_CFB8("aes-256-cfb8"),
    AES_256_CFB128("aes-256-cfb128"),
    RC4("rc4"),
    RC4_MD5("rc4-md5"),
    CHACHA20("chacha20"),
    SALSA20("salsa20"),
    CHACHA20_IETF("chacha20-ietf"),
    AES_128_GCM("aes-128-gcm"),
    AES_256_GCM("aes-256-gcm"),
    CHACHA20_IETF_POLY1305("chacha20-ietf-poly1305"),
    XCHACHA20_IETF_POLY1305("xchacha20-ietf-poly1305"),
    AES_128_PMAC_SIV("aes-128-pmac-siv"),
    AES_256_PMAC_SIV("aes-256-pmac-siv");

    companion object {
        fun fromString(input: String) = Ciphers.entries.first { it.label == input }
    }
}
