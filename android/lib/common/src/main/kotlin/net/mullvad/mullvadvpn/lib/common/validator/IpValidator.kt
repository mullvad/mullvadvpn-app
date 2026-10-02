package net.mullvad.mullvadvpn.lib.common.validator

import org.apache.commons.validator.routines.InetAddressValidator

@Suppress("MagicNumber")
class IpValidator {
    private val validator = InetAddressValidator.getInstance()

    fun isValidIpOrCidr(raw: String): Boolean = isValidIp(raw) || isValidCidr(raw)

    fun isValidIp(raw: String): Boolean =
        // Guard against '/' and '%' because InetAddressValidator.isValidInet6Address()
        // natively accepts IPv6 CIDR prefixes (/0..128) and zone IDs (%eth0)
        '/' !in raw && '%' !in raw && validator.isValid(raw)

    @Suppress("ReturnCount")
    fun isValidCidr(raw: String): Boolean {
        // Guard against zone IDs like %eth0
        if ('%' in raw) return false

        val parts = raw.split('/')
        if (parts.size != 2) return false

        val (ip, prefixStr) = parts
        val prefix = parsePrefixLength(prefixStr) ?: return false

        return when {
            validator.isValidInet4Address(ip) -> prefix in 0..32
            validator.isValidInet6Address(ip) -> prefix in 0..128
            else -> false
        }
    }

    // Matches  non-negative integers without leading zeros or '+' (e.g. "0", "24", "128")
    @Suppress("ReturnCount")
    private fun parsePrefixLength(raw: String): Int? {
        if (raw == "0") return 0
        if (raw.length !in 1..3 || raw[0] !in '1'..'9' || !raw.all { it in '0'..'9' }) return null
        return raw.toIntOrNull()
    }
}
