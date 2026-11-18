package net.mullvad.mullvadvpn.feature.home.impl.connect.connectioninfo

import net.mullvad.mullvadvpn.lib.model.FeatureIndicator

internal val ANTI_CENSORSHIP_FEATURE_INDICATORS =
    listOf(
        FeatureIndicator.ANTI_CENSORSHIP,
        FeatureIndicator.WIREGUARD_PORT,
        FeatureIndicator.LWO,
        FeatureIndicator.QUIC,
        FeatureIndicator.SHADOWSOCKS,
        FeatureIndicator.UDP_2_TCP,
    )
