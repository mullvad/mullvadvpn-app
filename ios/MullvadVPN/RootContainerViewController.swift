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

    override var childForStatusBarStyle: UIViewController? {
        return viewControllers.last
    }

    override var childForStatusBarHidden: UIViewController? {
        return viewControllers.last
    }

    // MARK: - View lifecycle

    override func awakeFromNib() {
        super.awakeFromNib()
    }

    // MARK: - Public

    override func allowedChildrenForUnwinding(from source: UIStoryboardUnwindSegueSource) -> [UIViewController] {
        let sourceViewController = childContaining(source)

        var allowedChildren = viewControllers
        allowedChildren.removeAll(where: { $0 == sourceViewController })

        return allowedChildren
    }

    func setViewControllers(_ newViewControllers: [UIViewController], animated: Bool, completion: CompletionHandler? = nil) {
        let currentTopViewController = viewControllers.last

        let viewControllersToRemove = viewControllers.filter { (viewController) -> Bool in
            return !newViewControllers.contains(viewController)
        }

        let viewControllersToAdd = newViewControllers.filter { (viewController) -> Bool in
            return !viewControllers.contains(viewController)
        }

        let finishTransition = {
            for child in viewControllersToRemove {
                self.removeChildFromContainer(child)
            }
            
            completion?()
        }

        // Add new child controllers
        for child in viewControllersToAdd {
            addChildIntoContainer(child)
        }

        // Hide all controllers except the current and the next top controllers
        let lastIndex = newViewControllers.count - 1
        for (index, child) in newViewControllers.enumerated() {
            let keepVisible = index == lastIndex || child == currentTopViewController

            child.view.isHidden = !keepVisible
        }
        
        viewControllers = newViewControllers

        if animated {
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

    private func addChildIntoContainer(_ child: UIViewController) {
        addChild(child)
        
        child.view.translatesAutoresizingMaskIntoConstraints = true
        child.view.autoresizingMask = [.flexibleWidth, .flexibleHeight]
        child.view.frame = view.bounds

        view.addSubview(child.view)
        child.didMove(toParent: self)
    }

    private func removeChildFromContainer(_ child: UIViewController) {
        child.willMove(toParent: nil)
        child.view.removeFromSuperview()
        child.removeFromParent()
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
