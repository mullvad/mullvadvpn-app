package net.mullvad.mullvadvpn.utils

import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.util.VoucherRegexHelper
import org.hamcrest.CoreMatchers.equalTo
import org.hamcrest.MatcherAssert.assertThat
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.junit.runners.Parameterized

private const val IS_ACCEPTED_FORMAT = true
private const val IS_UNACCEPTED_FORMAT = false

@RunWith(Parameterized::class)
class VoucherRegexHelperParameterizedTest(
    private val isValid: Boolean,
    private val voucher: String
) {
    @get:Rule val testCoroutineRule = TestCoroutineRule()

    @Test
    fun testVoucherFormat() {
        assertThat(VoucherRegexHelper.validate(voucher), equalTo(isValid))
    }

    companion object {
        @JvmStatic
        @Parameterized.Parameters
        fun data(): Collection<Array<Any>> =
            listOf(
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
