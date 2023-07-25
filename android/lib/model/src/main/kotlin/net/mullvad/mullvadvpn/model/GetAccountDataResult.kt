package net.mullvad.mullvadvpn.model

sealed class GetAccountDataResult {
    class Ok(val accountData: AccountData) : GetAccountDataResult()
    object InvalidAccount : GetAccountDataResult()
    object RpcError : GetAccountDataResult()
    object OtherError : GetAccountDataResult()
}
