package net.mullvad.mullvadvpn.lib.common.validator

import org.junit.jupiter.api.Assertions.assertFalse
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.params.ParameterizedTest
import org.junit.jupiter.params.provider.ValueSource

class IpValidatorTest {

    private val validator = IpValidator()

    @ParameterizedTest(name = "[{index}] \"{0}\"")
    @ValueSource(
        strings =
            [
                "192.168.1.1",
                "2001:db8::1",
                "::",
                "::1",
            ],
    )
    fun `ensure bare IPv4 and IPv6 addresses are valid IPs but not CIDRs`(input: String) {
        assertTrue(validator.isValidIp(input))
        assertFalse(validator.isValidCidr(input))
        assertTrue(validator.isValidIpOrCidr(input))
    }

    @ParameterizedTest(name = "[{index}] \"{0}\"")
    @ValueSource(
        strings =
            [
                "192.168.1.0/24",
                "2001:db8::/32",
            ],
    )
    fun `ensure isValidIp rejects CIDR addresses while CIDR checks accept them`(input: String) {
        assertFalse(validator.isValidIp(input))
        assertTrue(validator.isValidCidr(input))
        assertTrue(validator.isValidIpOrCidr(input))
    }

    @ParameterizedTest(name = "[{index}] \"{0}\"")
    @ValueSource(
        strings =
            [
                "0.0.0.0/0",
                "192.168.1.0/32",
                "::/0",
                "2001:db8::/128",
            ],
    )
    fun `ensure valid boundary CIDRs are accepted`(input: String) {
        assertFalse(validator.isValidIp(input))
        assertTrue(validator.isValidCidr(input))
        assertTrue(validator.isValidIpOrCidr(input))
    }

    @ParameterizedTest(name = "[{index}] \"{0}\"")
    @ValueSource(
        strings =
            [
                "fe80::1%eth0",
                "fe80::1%eth0/64",
            ],
    )
    fun `ensure zone IDs are rejected everywhere`(input: String) {
        assertFalse(validator.isValidIp(input))
        assertFalse(validator.isValidCidr(input))
        assertFalse(validator.isValidIpOrCidr(input))
    }

    @ParameterizedTest(name = "[{index}] \"{0}\"")
    @ValueSource(
        strings =
            [
                "192.168.1.0/08",
                "2001:db8::/08",
                "192.168.1.0/+8",
                "192.168.1.0/-1",
                "192.168.1.0/33",
                "2001:db8::/129",
                "192.168.1.0/",
                "/",
                "192.168.1.0/24/1",
                "/24",
            ],
    )
    fun `ensure invalid CIDR prefixes are rejected everywhere`(input: String) {
        assertFalse(validator.isValidIp(input))
        assertFalse(validator.isValidCidr(input))
        assertFalse(validator.isValidIpOrCidr(input))
    }

    @ParameterizedTest(name = "[{index}] \"{0}\"")
    @ValueSource(
        strings =
            [
                "",
                " ",
                " 192.168.1.1",
                "192.168.1.1 ",
            ],
    )
    fun `ensure empty and whitespace inputs are rejected everywhere`(input: String) {
        assertFalse(validator.isValidIp(input))
        assertFalse(validator.isValidCidr(input))
        assertFalse(validator.isValidIpOrCidr(input))
    }
}
