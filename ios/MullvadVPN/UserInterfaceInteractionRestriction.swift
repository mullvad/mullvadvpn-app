//
//  UserInterfaceInteractionRestriction.swift
//  MullvadVPN
//
//  Created by pronebird on 20/03/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// A protocol describing a common interface for the implementations of user interaction restriction
protocol UserInterfaceInteractionRestrictionProtocol {
    /// Increase the user interface interaction restrictions
    func increase(animated: Bool)

    /// Decrease the user interface interaction restrictions
    func decrease(animated: Bool)
}

/// A counter based user interface interaction restriction implementation
class UserInterfaceInteractionRestriction: UserInterfaceInteractionRestrictionProtocol {
    typealias Action = (_ enableUserInteraction: Bool, _ animated: Bool) -> Void

    private let action: Action
    private var counter: UInt = 0

    init(action: @escaping Action) {
        self.action = action
    }

    func increase(animated: Bool) {
        DispatchQueue.main.async {
            if self.counter == 0 {
                self.action(false, animated)
            }
            self.counter += 1
        }
    }

    func decrease(animated: Bool) {
        DispatchQueue.main.async {
            guard self.counter > 0 else { return }

            self.counter -= 1
            if self.counter == 0 {
                self.action(true, animated)
            }
        }
    }
}

/// A user interface restriction implementation that simply combines multiple child restrictions
/// into one and automatically forwards all calls to them in the order in which they are given to
/// the initializer.
class CompoundUserInterfaceInteractionRestriction: UserInterfaceInteractionRestrictionProtocol {
    private let restrictions: [UserInterfaceInteractionRestrictionProtocol]

    init(restrictions: [UserInterfaceInteractionRestrictionProtocol]) {
        self.restrictions = restrictions
    }

    func decrease(animated: Bool) {
        restrictions.forEach { $0.decrease(animated: animated) }
    }

    func increase(animated: Bool) {
        restrictions.forEach { $0.increase(animated: animated) }
    }
}
