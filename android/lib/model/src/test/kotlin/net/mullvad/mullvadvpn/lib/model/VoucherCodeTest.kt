package net.mullvad.mullvadvpn.lib.model

import kotlin.test.assertEquals
import kotlin.test.assertTrue
import org.junit.jupiter.api.Test

class VoucherCodeTest {
    @Test
    fun `parsing a too short voucher code should return TooShort`() {
        val input = "mycode"
        val result = VoucherCode.fromString(input)

        assertTrue(result.isLeft())
        assertEquals(ParseVoucherCodeError.TooShort(input), result.leftOrNull())
    }

    @Test
    fun `numbers only should not be allowed`() {
        val input = "1234123412341234"
        val result = VoucherCode.fromString(input)

        assertTrue(result.isLeft())
        assertEquals(ParseVoucherCodeError.AllDigit(input), result.leftOrNull())
    }

    @Test
    fun `number only input when too short should return TooShort`() {
        val input = "123412341234"
        val result = VoucherCode.fromString(input)

        assertTrue(result.isLeft())
        assertEquals(ParseVoucherCodeError.TooShort(input), result.leftOrNull())
    }
}
