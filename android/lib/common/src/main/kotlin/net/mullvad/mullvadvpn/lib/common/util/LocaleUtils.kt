@file:Suppress("MatchingDeclarationName")

package net.mullvad.mullvadvpn.lib.common.util

// To get the correct sort order for the country names in Japanese we need to keep
// a mapping of the country names that contain kanji to their kana reading, and sort on the reading.
object JapaneseSort {
    private val kanjiCountryReadingMap =
        mapOf(
            "日本" to "ニホン",
            "中国" to "チュウゴク",
            "韓国" to "カンコク",
            "北朝鮮" to "キタチョウセン",
            "台湾" to "タイワン",
            "香港" to "ホンコン",
            "英国" to "エイコク",
            "米国" to "ベイコク",
            "南アフリカ" to "ミナミアフリカ",
            "中央アフリカ" to "チュウオウアフリカ",
            "南スーダン" to "ミナミスーダン",
            "北マケドニア" to "キタマケドニア",
            "東ティモール" to "ヒガシティモール",
            "赤道ギニア" to "セキドウギニア",
            "西サハラ" to "ニシサハラ",
            "アラブ首長国連邦" to "アラブシュチョウコクレンポウ",
            "ドミニカ共和国" to "ドミニカキョウワコク",
            "アメリカ合衆国" to "アメリカガッシュウコク",
            "南アフリカ共和国" to "ミナミアフリカキョウワコク",
            "中央アフリカ共和国" to "チュウオウアフリカキョウワコク",
        )

    fun getSortKey(countryName: String): String {
        return kanjiCountryReadingMap[countryName] ?: countryName
    }
}
