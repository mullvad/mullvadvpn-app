package net.mullvad.mullvadvpn.utils

import java.util.stream.Stream
import kotlin.test.assertEquals
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.util.VoucherRegexHelper
import org.junit.jupiter.api.extension.ExtendWith
import org.junit.jupiter.params.ParameterizedTest
import org.junit.jupiter.params.provider.MethodSource

private const val IS_ACCEPTED_FORMAT = true
private const val IS_UNACCEPTED_FORMAT = false

@ExtendWith(TestCoroutineRule::class)
class VoucherRegexHelperParameterizedTest {

    @ParameterizedTest(name = "Voucher format {1} should be valid: {0}")
    @MethodSource("data")
    fun testVoucherFormat(isValid: Boolean, voucher: String) {
        assertEquals(VoucherRegexHelper.validate(voucher), isValid)
    }

    companion object {
        @JvmStatic
        fun data(): Stream<Array<Any>> =
            Stream.of(
                arrayOf(IS_ACCEPTED_FORMAT, "1"),
                arrayOf(IS_ACCEPTED_FORMAT, "a"),
                arrayOf(IS_ACCEPTED_FORMAT, "A"),
                arrayOf(IS_ACCEPTED_FORMAT, "AAAA"),
                arrayOf(IS_ACCEPTED_FORMAT, "AAAABBBB11112222"),
                arrayOf(IS_ACCEPTED_FORMAT, "AAAA BBBB 1111 2222"),
                arrayOf(IS_ACCEPTED_FORMAT, "AAAA-AAAA-1111-2222\r"),
                arrayOf(IS_ACCEPTED_FORMAT, "AAAA-AAAA-1111-2222\n"),
                arrayOf(IS_UNACCEPTED_FORMAT, "@"),
                arrayOf(IS_UNACCEPTED_FORMAT, "AAAABBBBCCCCDDDD\t"),
                arrayOf(IS_UNACCEPTED_FORMAT, "AAAA_BBBB_CCCC_DDDD")
            )
    }
}
