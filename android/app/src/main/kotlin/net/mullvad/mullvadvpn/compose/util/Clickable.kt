package net.mullvad.mullvadvpn.compose.util

// Based on:
// https://stackoverflow.com/questions/69901608/how-to-disable-simultaneous-clicks-on-multiple-items-in-jetpack-compose-list-c
fun singleClick(onClick: () -> Unit): () -> Unit {
    var latest: Long = 0
    return {
        val now = System.currentTimeMillis()
        if (now - latest >= CLICK_TIMEOUT) {
            onClick()
            latest = now
        }
    }
}

private const val CLICK_TIMEOUT = 300
