//
//  MultihopSettings.swift
//  MullvadSettings
//
//  Created by Mojgan on 2024-04-26.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

public protocol MultihopPropagation {
    typealias MultihopHandler = (MultihopState) -> Void
    var onNewMultihop: MultihopHandler? { get set }
}

public protocol MultihopObserver: AnyObject {
    func multihop(_ object: MultihopPropagation, didUpdateMultihop state: MultihopState)
}

public class MultihopObserverBlock: MultihopObserver {
    public typealias DidUpdateMultihopHandler = (MultihopPropagation, MultihopState) -> Void
    public var onNewState: DidUpdateMultihopHandler

    public init(didUpdateMultihop: @escaping DidUpdateMultihopHandler) {
        self.onNewState = didUpdateMultihop
    }

    public func multihop(_ object: MultihopPropagation, didUpdateMultihop state: MultihopState) {
        self.onNewState(object, state)
    }
}

public final class MultihopStateListener: MultihopPropagation {
    public var onNewMultihop: MultihopHandler?

    public init(onNewMultihop: MultihopHandler? = nil) {
        self.onNewMultihop = onNewMultihop
    }
}

public class MultihopUpdater {
    /// Observers.
    private let observerList = ObserverList<MultihopObserver>()
    private var listener: MultihopPropagation

    public init(listener: MultihopPropagation) {
        self.listener = listener
        self.listener.onNewMultihop = { [weak self] state in
            guard let self else { return }
            self.observerList.notify {
                $0.multihop(listener, didUpdateMultihop: state)
            }
        }
    }

    // MARK: - Multihop observations

    public func addObserver(_ observer: MultihopObserver) {
        observerList.append(observer)
    }

    public func removeObserver(_ observer: MultihopObserver) {
        observerList.remove(observer)
    }
}

/// Whether Multi-hop is enabled
public enum MultihopState: Codable {
    case on
    case off
}
