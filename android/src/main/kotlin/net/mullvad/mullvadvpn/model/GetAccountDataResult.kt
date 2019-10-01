package net.mullvad.mullvadvpn.model

sealed class GetAccountDataResult {
    class Ok(val accountData: AccountData) : GetAccountDataResult()
    class InvalidAccount : GetAccountDataResult()
    class RpcError : GetAccountDataResult()
    class OtherError : GetAccountDataResult()
}
