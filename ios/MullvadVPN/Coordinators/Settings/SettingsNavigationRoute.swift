//
//  SettingsNavigationRoute.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2026-03-10.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

/// Settings navigation route.
enum SettingsNavigationRoute: Equatable {
    /// The route that's always displayed first upon entering settings.
    case root

    /// VPN settings.
    case vpnSettings

    /// Problem report.
    case problemReport

    /// FAQ section displayed as a modal safari browser.
    case faq

    /// API access route.
    case apiAccess

    /// changelog route.
    case changelog

    /// Multihop route.
    case multihop

    /// DAITA route.
    case daita

    /// Language route.
    case language

    /// Notification settings route.
    case notificationSettings

    /// IAN route.
    case includeAllNetworks
}
