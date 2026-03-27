package net.mullvad.mullvadvpn.feature.settings.impl.server

import org.jsoup.Jsoup

suspend fun download() = Jsoup.connect("https://mullvad.net/en/help/faq").get()
