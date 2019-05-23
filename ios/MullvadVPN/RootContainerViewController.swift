//
//  RootContainerViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 25/05/2019.
//  Copyright © 2019 Amagicom AB. All rights reserved.
//

import UIKit

/// A root container class that primarily handles the unwind storyboard segues on log out
class RootContainerViewController: UIViewController {

    typealias CompletionHandler = () -> Void

    private var viewControllers = [UIViewController]()

    private var topViewController: UIViewController? {
        return viewControllers.last
    }

    override var childForStatusBarStyle: UIViewController? {
        return topViewController
    }

    override var childForStatusBarHidden: UIViewController? {
        return topViewController
    }

    override var shouldAutomaticallyForwardAppearanceMethods: Bool {
        return false
    }

    // MARK: - View lifecycle

    override func viewWillAppear(_ animated: Bool) {
        super.viewWillAppear(animated)

        topViewController?.beginAppearanceTransition(true, animated: animated)
    }

    override func viewDidAppear(_ animated: Bool) {
        super.viewDidAppear(animated)

        topViewController?.endAppearanceTransition()
    }

    override func viewWillDisappear(_ animated: Bool) {
        super.viewWillDisappear(animated)

        topViewController?.beginAppearanceTransition(false, animated: animated)
    }

    override func viewDidDisappear(_ animated: Bool) {
        super.viewDidDisappear(animated)

        topViewController?.endAppearanceTransition()
    }

    // MARK: - Public

    override func allowedChildrenForUnwinding(from source: UIStoryboardUnwindSegueSource) -> [UIViewController] {
        let sourceViewController = childContaining(source)

        var allowedChildren = viewControllers
        allowedChildren.removeAll(where: { $0 == sourceViewController })

        return allowedChildren
    }

    func setViewControllers(_ newViewControllers: [UIViewController], animated: Bool, completion: CompletionHandler? = nil) {
        // Dot not handle appearance events when the container itself is not visible
        let shouldHandleAppearanceEvents = view.window != nil

        // Animations won't run when the container is not visible, so prevent them
        let shouldAnimate = animated && shouldHandleAppearanceEvents

        let currentTopViewController = topViewController
        let nextTopViewController = newViewControllers.last

        let viewControllersToRemove = viewControllers.filter { (viewController) -> Bool in
            return !newViewControllers.contains(viewController)
        }

        let viewControllersToAdd = newViewControllers.filter { (viewController) -> Bool in
            return !viewControllers.contains(viewController)
        }

        let finishTransition = {
            // Notify the added controllers that they finished a transition into the container
            for child in viewControllersToAdd {
                child.didMove(toParent: self)
            }

            // Remove the controllers that transitioned out of the container
            // The call to removeFromParent() automatically calls child.didMove()
            for child in viewControllersToRemove {
                child.view.removeFromSuperview()
                child.removeFromParent()
            }

            // Finish appearance transition
            if shouldHandleAppearanceEvents {
                currentTopViewController?.endAppearanceTransition()
                if currentTopViewController != nextTopViewController {
                    nextTopViewController?.endAppearanceTransition()
                }
            }

            completion?()
        }

        // Add new child controllers. The call to addChild() automatically calls child.willMove()
        for child in viewControllersToAdd {
            addChild(child)
            addChildView(child.view)
        }

        // Notify the controllers that they will transition out of the container
        for child in viewControllersToRemove {
            child.willMove(toParent: nil)
        }

        // Hide all controllers except the current and the next top controller
        let lastIndex = newViewControllers.count - 1
        for (index, child) in newViewControllers.enumerated() {
            let keepVisible = index == lastIndex || child == currentTopViewController

            child.view.isHidden = !keepVisible
        }

        viewControllers = newViewControllers

        // Begin appearance transition
        if shouldHandleAppearanceEvents {
            currentTopViewController?.beginAppearanceTransition(false, animated: shouldAnimate)
            if currentTopViewController != nextTopViewController {
                nextTopViewController?.beginAppearanceTransition(true, animated: shouldAnimate)
            }
        }

        if shouldAnimate {
            CATransaction.begin()
            CATransaction.setCompletionBlock {
                finishTransition()
            }

            let transition = CATransition()
            transition.duration = 0.25
            transition.type = .fade

            view.layer.add(transition, forKey: "transition")

            CATransaction.commit()
        } else {
            finishTransition()
        }
    }

    func pushViewController(_ viewController: UIViewController, animated: Bool) {
        var newViewControllers = viewControllers.filter({ $0 != viewController })
        newViewControllers.append(viewController)

        setViewControllers(newViewControllers, animated: animated)
    }

    // MARK: - Storyboard segue handling

    override func unwind(for unwindSegue: UIStoryboardSegue, towards subsequentVC: UIViewController) {
        let index = viewControllers.firstIndex(of: subsequentVC)!
        let newViewControllers = Array(viewControllers.prefix(through: index))

        let animated = UIView.areAnimationsEnabled

        setViewControllers(newViewControllers, animated: animated)
    }

    // MARK: - Private

    private func addChildView(_ childView: UIView) {
        childView.translatesAutoresizingMaskIntoConstraints = true
        childView.autoresizingMask = [.flexibleWidth, .flexibleHeight]
        childView.frame = view.bounds

        view.addSubview(childView)
    }
    
}

class RootContainerPushSegue: UIStoryboardSegue {
    override func perform() {
        let container = source.rootContainerController!
        let animated = UIView.areAnimationsEnabled

        container.pushViewController(destination, animated: animated)
    }
}

extension UIViewController {

    var rootContainerController: RootContainerViewController? {
        var viewController: UIViewController? = parent

        while viewController != nil {
            if let container = viewController as? RootContainerViewController {
                return container
            }

            viewController = viewController?.parent
        }

        return nil
    }

}
