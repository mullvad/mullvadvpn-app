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
open class Coordinator: NSObject {
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
    public var childCoordinators: [Coordinator] {
        _children
    }

    /// Parent coordinator.
    public var parent: Coordinator? {
        _parent
    }

    // MARK: - Children

    /**
     Add child coordinator.

     Adding the same coordinator twice is a no-op.
     */
    public func addChild(_ child: Coordinator) {
        guard !_children.contains(child) else { return }

        _children.append(child)
        child._parent = self

        logger.trace("Add child \(child)")
    }

    /**
     Remove child coordinator.

     Removing coordinator that's no longer a child of this coordinator is a no-op.
     */
    public func removeChild(_ child: Coordinator) {
        guard let index = _children.firstIndex(where: { $0 == child }) else { return }

        _children.remove(at: index)
        child._parent = nil

        logger.trace("Remove child \(child)")
    }

    /**
     Remove coordinator from its parent.
     */
    public func removeFromParent() {
        _parent?.removeChild(self)
    }
}

/**
 Protocol describing coordinators that can be presented using modal presentation.
 */
public protocol Presentable: Coordinator {
    /**
     View controller that is presented modally. It's expected it to be the topmost view controller
     managed by coordinator.
     */
    var presentedViewController: UIViewController { get }
}

/**
 Protocol describing coordinators that provide modal presentation context.
 */
public protocol Presenting: Coordinator {
    /**
     View controller providing modal presentation context.
     */
    var presentationContext: UIViewController { get }
}

extension Presenting where Self: Presentable {
    /**
     View controller providing modal presentation context.
     */
    public var presentationContext: UIViewController {
        return presentedViewController
    }
}

extension Presenting {
    /**
     Present child coordinator.

     Automatically adds child and removes it upon interactive dismissal.
     */
    public func presentChild(
        _ child: some Presentable,
        animated: Bool,
        configuration: ModalPresentationConfiguration = ModalPresentationConfiguration(),
        completion: (() -> Void)? = nil
    ) {
//        assert(
//            presentationContext.presentedViewController == nil,
//            "Presenting context (\(presentationContext)) is already presenting another controller."
//        )

        var configuration = configuration

        configuration.notifyInteractiveDismissal { [weak child] in
            guard let child else { return }

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
    public func dismiss(animated: Bool, completion: (() -> Void)? = nil) {
        removeFromParent()

        presentedViewController.dismiss(animated: animated, completion: completion)
    }

    /**
     Add block based observer triggered if coordinator is dismissed via user interaction.
     */
    public func onInteractiveDismissal(_ handler: @escaping (Coordinator) -> Void) {
        interactiveDismissalObservers.append(handler)
    }
}
