//
//  MultihopState.swift
//  MullvadSettings
//
//  Created by Mojgan on 2024-04-26.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

public typealias MultihopState = MultihopStateV2

public protocol MultihopStateMigrating {
    func upgradeToNextVersion() -> any MultihopStateMigrating
}

/// In which circumstances Multihop is enabled
public enum MultihopStateV2: CustomStringConvertible, CaseIterable, Codable, Sendable {
    case whenNeeded
    case always
    case never

    public var isWhenNeeded: Bool {
        self == .whenNeeded
    }

    public var isAlways: Bool {
        self == .always
    }

    public var isNever: Bool {
        self == .never
    }

    public var description: String {
        switch self {
        case .always: NSLocalizedString("Always", comment: "")
        case .whenNeeded: NSLocalizedString("When needed", comment: "")
        case .never: NSLocalizedString("Never", comment: "")
        }
    }
}

extension MultihopStateV2: MultihopStateMigrating {
    public func upgradeToNextVersion() -> any MultihopStateMigrating {
        self
    }
}

/// #MARK: versions of MultihopState used in previous versions of the settings

public enum MultihopStateV1: Codable, Sendable {
    case on
    case off
}

extension MultihopStateV1: MultihopStateMigrating {
    public func upgradeToNextVersion() -> any MultihopStateMigrating {
        switch self {
        case .on: MultihopStateV2.always
        case .off: MultihopStateV2.never
        }
    }
}
