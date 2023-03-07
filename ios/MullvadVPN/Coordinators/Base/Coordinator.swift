//
//  Coordinator.swift
//  MullvadVPN
//
//  Created by pronebird on 27/01/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import MullvadLogging
import UIKit

/**
 Base coordinator class.

 Coordinators help to abstract the navigation and business logic from view controllers making them
 more manageable and reusable.
 */
class Coordinator: NSObject {
    /// Private trace log.
    private lazy var logger = Logger(label: "\(Self.self)")

    /// Weak reference to parent coordinator.
    private weak var _parent: Coordinator?

    /// Mutable collection of child coordinators.
    private var _children: [Coordinator] = []

    /// Modal presentation configuration assigned on presented coordinator.
    fileprivate var modalConfiguration: ModalPresentationConfiguration?

    /// An array of blocks that are invoked upon interactive dismissal.
    fileprivate var interactiveDismissalObservers: [(Coordinator) -> Void] = []

    /// Child coordinators.
    var childCoordinators: [Coordinator] {
        return _children
    }

    /// Parent coordinator.
    var parent: Coordinator? {
        return _parent
    }

    // MARK: - Children

    /**
     Add child coordinator.

     Adding the same coordinator twice is a no-op.
     */
    func addChild(_ child: Coordinator) {
        guard !_children.contains(child) else { return }

        _children.append(child)
        child._parent = self

        logger.trace("Add child \(child)")
    }

    /**
     Remove child coordinator.

     Removing coordinator that's no longer a child of this coordinator is a no-op.
     */
    func removeChild(_ child: Coordinator) {
        guard let index = _children.firstIndex(where: { $0 == child }) else { return }

        _children.remove(at: index)
        child._parent = nil

        logger.trace("Remove child \(child)")
    }

    /**
     Remove coordinator from its parent.
     */
    func removeFromParent() {
        _parent?.removeChild(self)
    }
}

/**
 Protocol describing coordinators that can be presented using modal presentation.
 */
protocol Presentable: Coordinator {
    /**
     View controller that is presented modally. It's expected it to be the top-most view controller
     managed by coordinator.
     */
    var presentedViewController: UIViewController { get }
}

/**
 Protocol describing coordinators that provide modal presentation context.
 */
protocol Presenting: Coordinator {
    /**
     View controller providing modal presentation context.
     */
    var presentationContext: UIViewController { get }
}

extension Presenting {
    /**
     Present child coordinator.

     Automatically adds child and removes it upon interactive dismissal.
     */
    func presentChild<T: Presentable>(
        _ child: T,
        animated: Bool,
        configuration: ModalPresentationConfiguration = ModalPresentationConfiguration(),
        completion: (() -> Void)? = nil
    ) {
        var configuration = configuration

        configuration.notifyInteractiveDismissal { [weak child] in
            guard let child = child else { return }

            child.modalConfiguration = nil
            child.removeFromParent()

            let observers = child.interactiveDismissalObservers
            child.interactiveDismissalObservers = []

            for observer in observers {
                observer(child)
            }
        }

        configuration.apply(to: child.presentedViewController)

        child.modalConfiguration = configuration

        addChild(child)

        presentationContext.present(
            child.presentedViewController,
            animated: animated,
            completion: completion
        )
    }
}

extension Presentable {
    /**
     Dismiss this coordinator.

     Automatically removes itself from parent.
     */
    func dismiss(animated: Bool, completion: (() -> Void)? = nil) {
        removeFromParent()

        presentedViewController.dismiss(animated: animated, completion: completion)
    }

    /**
     Add block based observer triggered if coordinator is dismissed via user interaction.
     */
    func onInteractiveDismissal(_ handler: @escaping (Coordinator) -> Void) {
        interactiveDismissalObservers.append(handler)
    }
}
