//
//  UITestSettingsResetPolicy.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-03-02.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//
public protocol ResettableKey: Hashable, Codable, Sendable, CaseIterable {}

public enum UITestResetPolicy<Key: ResettableKey>: Sendable {
    case all
    case none
    case only(Set<Key>)
    case allExcept(Set<Key>)
}
extension UITestResetPolicy: Codable {

    private enum CodingKeys: String, CodingKey {
        case type
        case keys
    }

    private enum PolicyType: String, Codable {
        case all
        case none
        case only
        case allExcept
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)

        switch self {
        case .all:
            try container.encode(PolicyType.all, forKey: .type)

        case .none:
            try container.encode(PolicyType.none, forKey: .type)

        case .only(let keys):
            try container.encode(PolicyType.only, forKey: .type)
            try container.encode(keys, forKey: .keys)

        case .allExcept(let keys):
            try container.encode(PolicyType.allExcept, forKey: .type)
            try container.encode(keys, forKey: .keys)
        }
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        let type = try container.decode(PolicyType.self, forKey: .type)

        switch type {
        case .all:
            self = .all

        case .none:
            self = .none

        case .only:
            let keys = try container.decode(Set<Key>.self, forKey: .keys)
            self = .only(keys)

        case .allExcept:
            let keys = try container.decode(Set<Key>.self, forKey: .keys)
            self = .allExcept(keys)
        }
    }
}

public typealias UITestSettingsResetPolicy = UITestResetPolicy<UITestSettingsKey>
public typealias UITestAppPreferencesPolicy = UITestResetPolicy<UITestAppPreferencesKey>

extension UITestSettingsKey: ResettableKey {}
extension UITestAppPreferencesKey: ResettableKey {}

public enum UITestSettingsKey: String, CaseIterable, Codable, Sendable {
    case settings
    case ipOverrides
    case customRelayLists
    case recentConnections
}

public enum UITestAppPreferencesKey: String, CaseIterable, Codable, Sendable {
    case hasDoneFirstTimeLaunch = "hasFinishedFirstTimeLaunch"
    case hasDoneFirstTimeLogin
    case isShownOnboarding
    case isAgreedToTermsOfService
    case lastSeenChangeLogVersion
    case lastVersionCheck
    case isNotificationPermissionAsked
    case notificationSettings
    case includeAllNetworksConsent
}

extension UITestResetPolicy {
    public func resolvedKeys() -> Set<Key> {
        switch self {
        case .all:
            return Set(Key.allCases)

        case .none:
            return []

        case .only(let keys):
            return keys

        case .allExcept(let excluded):
            return Set(Key.allCases).subtracting(excluded)
        }
    }

    /// Convenience check
    public func shouldReset(_ key: Key) -> Bool {
        resolvedKeys().contains(key)
    }
}
