package net.mullvad.mullvadvpn.lib.model

import de.infix.testBalloon.framework.core.testSuite
import kotlin.test.assertEquals
import kotlin.test.assertTrue

val VoucherCodeTestSuite by testSuite("VoucherCode tests") {
    test("parsing a too short voucher code should return TooShort") {
        val input = "mycode"
        val result = VoucherCode.fromString(input)

        assertTrue(result.isLeft())
        assertEquals(ParseVoucherCodeError.TooShort(input), result.leftOrNull())
    }

    test("numbers only should not be allowed") {
        val input = "1234123412341234"
        val result = VoucherCode.fromString(input)

        assertTrue(result.isLeft())
        assertEquals(ParseVoucherCodeError.AllDigit(input), result.leftOrNull())
    }

    test("number only input when too short should return TooShort") {
        val input = "123412341234"
        val result = VoucherCode.fromString(input)

        assertTrue(result.isLeft())
        assertEquals(ParseVoucherCodeError.TooShort(input), result.leftOrNull())
    }
}
