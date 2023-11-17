package net.mullvad.mullvadvpn.utils

import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.util.VoucherRegexHelper
import org.hamcrest.CoreMatchers.equalTo
import org.hamcrest.MatcherAssert.assertThat
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.junit.runners.Parameterized

@RunWith(Parameterized::class)
class RedeemVoucherHelperTest(private val isValid: Boolean, private val voucher: String) {
    @get:Rule val testCoroutineRule = TestCoroutineRule()

    companion object {
        @JvmStatic
        @Parameterized.Parameters
        fun data(): Collection<Array<Any>> =
            listOf(
                arrayOf(true, acceptable_inputs_for_voucher[0]),
                arrayOf(true, acceptable_inputs_for_voucher[1]),
                arrayOf(true, acceptable_inputs_for_voucher[2]),
                arrayOf(true, acceptable_inputs_for_voucher[3]),
                arrayOf(true, acceptable_inputs_for_voucher[4]),
                arrayOf(true, acceptable_inputs_for_voucher[5]),
                arrayOf(true, acceptable_inputs_for_voucher[6]),
                arrayOf(true, acceptable_inputs_for_voucher[7]),
                arrayOf(false, non_acceptable_inputs_for_voucher[0]),
                arrayOf(false, non_acceptable_inputs_for_voucher[1]),
                arrayOf(false, non_acceptable_inputs_for_voucher[2])
            )

        private val acceptable_inputs_for_voucher =
            arrayOf(
                "1",
                "a",
                "A",
                "AAAA",
                "AAAABBBB11112222",
                "AAAA BBBB 1111 2222",
                "AAAA-AAAA-1111-2222\r",
                "AAAA-AAAA-1111-2222\n",
            )
        private val non_acceptable_inputs_for_voucher =
            arrayOf(
                "@",
                "AAAABBBBCCCCDDDD\t",
                "AAAA_BBBB_CCCC_DDDD",
            )
    }

    @Test
    fun shouldReturnExpectedResultForVoucherCheck() {
        assertThat(VoucherRegexHelper.validate(voucher), equalTo(isValid))
    }
}
