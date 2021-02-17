package net.mullvad.mullvadvpn.model

sealed class VoucherSubmissionResult {
    class Ok(val submission: VoucherSubmission) : VoucherSubmissionResult()

    class InvalidVoucher : VoucherSubmissionResult() {
        companion object {
            @JvmStatic
            val INSTANCE = InvalidVoucher()
        }
    }

    class VoucherAlreadyUsed : VoucherSubmissionResult() {
        companion object {
            @JvmStatic
            val INSTANCE = VoucherAlreadyUsed()
        }
    }

    class RpcError : VoucherSubmissionResult() {
        companion object {
            @JvmStatic
            val INSTANCE = RpcError()
        }
    }

    class OtherError : VoucherSubmissionResult() {
        companion object {
            @JvmStatic
            val INSTANCE = OtherError()
        }
    }
}
