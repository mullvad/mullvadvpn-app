//
//  UserInterfaceInteractionRestriction.swift
//  MullvadVPN
//
//  Created by pronebird on 20/03/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Combine
import Foundation

/// A protocol describing a common interface for the implementations of user interaction restriction
protocol UserInterfaceInteractionRestrictionProtocol {
    /// Raise the user interface interaction restrictions
    func lift(animated: Bool)

    /// Lift the user interface interaction restrictions
    func raise(animated: Bool)
}

/// A counter based user interface interaction restriction implementation
class UserInterfaceInteractionRestriction<S: Scheduler>
    : UserInterfaceInteractionRestrictionProtocol
{
    typealias Action = (_ disableUserInteraction: Bool, _ animated: Bool) -> Void

    private let action: Action
    private let scheduler: S
    private var counter: UInt = 0

    init(scheduler: S, action: @escaping Action) {
        self.action = action
        self.scheduler = scheduler
    }

    func raise(animated: Bool) {
        scheduler.schedule {
            if self.counter == 0 {
                self.action(false, animated)
            }
            self.counter += 1
        }
    }

    func lift(animated: Bool) {
        scheduler.schedule {
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

    func lift(animated: Bool) {
        restrictions.forEach { $0.lift(animated: animated) }
    }

    func raise(animated: Bool) {
        restrictions.forEach { $0.raise(animated: animated) }
    }
}

extension Publisher {
    func restrictUserInterfaceInteraction(
        with restriction: UserInterfaceInteractionRestrictionProtocol,
        animated: Bool
    ) -> Publishers.HandleEvents<Self>
    {
        return handleEvents(receiveSubscription: { _ in
            restriction.raise(animated: animated)
        }, receiveCompletion: { _ in
            restriction.lift(animated: animated)
        }, receiveCancel: { () in
            restriction.lift(animated: animated)
        })
    }
}
