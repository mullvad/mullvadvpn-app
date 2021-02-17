package net.mullvad.mullvadvpn.model

sealed class GetAccountDataResult {
    class Ok(val accountData: AccountData) : GetAccountDataResult()

    class InvalidAccount : GetAccountDataResult() {
        companion object {
            @JvmStatic
            val INSTANCE = InvalidAccount()
        }
    }

    class RpcError : GetAccountDataResult() {
        companion object {
            @JvmStatic
            val INSTANCE = RpcError()
        }
    }

    class OtherError : GetAccountDataResult() {
        companion object {
            @JvmStatic
            val INSTANCE = OtherError()
        }
    }
}
