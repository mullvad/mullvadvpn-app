package net.mullvad.mullvadvpn.util

interface ListenableScrollableView {
    val horizontalScrollOffset: Int
    val verticalScrollOffset: Int

    var onScrollListener: ((Int, Int, Int, Int) -> Unit)?
}
