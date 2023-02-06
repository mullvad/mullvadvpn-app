//
//  ModalRootAdaptivePresentationDelegate.swift
//  MullvadVPN
//
//  Created by pronebird on 19/01/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

/**

 Adaptive presentation delegate for `SceneDelegate.modalRootContainer` used for presenting
 the login flow on iPad.

 The primary purpose of this class is to swap between fullscreen and formsheet presentation based
 on horizontal size class and make settings (cog) accessible even when parent root is overlayed with
 modal root.

 Unlike iPhone where only one `RootContainerViewController` is used and behaves very much like
 navigation controller, iPad uses two of such controllers defined as parent and (think child) modal
 within this class.

 ## iPhone view controller hierarchy

 - UIWindow
   - RootContainerViewController
     - LoginViewController
     - etc.

 ## iPad view controller hierarchy

 - UIWindow
   - RootContainerViewController (parent)
     - UISplitViewController
       - TunnelViewController
       - SelectLocationViewController
     - RootContainerViewController (child [modal])
       - LoginViewController
       - etc.

 */
final class ModalRootAdaptivePresentationDelegate: NSObject,
    UIAdaptivePresentationControllerDelegate
{
    let parentRootContainer: RootContainerViewController
    let modalRootContainer: RootContainerViewController

    init(
        parentRootContainer: RootContainerViewController,
        modalRootContainer: RootContainerViewController
    ) {
        self.parentRootContainer = parentRootContainer
        self.modalRootContainer = modalRootContainer

        super.init()

        NotificationCenter.default.addObserver(
            self,
            selector: #selector(dismissalTransitionDidEnd(_:)),
            name: UIPresentationController.dismissalTransitionDidEndNotification,
            object: modalRootContainer
        )
    }

    private func finishPresentation() {
        parentRootContainer.removeSettingsButtonFromPresentationContainer()
    }

    func adaptivePresentationStyle(
        for controller: UIPresentationController,
        traitCollection: UITraitCollection
    ) -> UIModalPresentationStyle {
        return traitCollection.horizontalSizeClass == .regular ? .formSheet : .fullScreen
    }

    func presentationController(
        _ presentationController: UIPresentationController,
        willPresentWithAdaptiveStyle style: UIModalPresentationStyle,
        transitionCoordinator: UIViewControllerTransitionCoordinator?
    ) {
        // The style is set to none when adaptive presentation is not changing.
        let actualStyle: UIModalPresentationStyle = style == .none
            ? presentationController.presentedViewController.modalPresentationStyle
            : style

        // Force hide header bar in .formSheet presentation and show it in .fullScreen presentation
        modalRootContainer.setOverrideHeaderBarHidden(actualStyle == .formSheet, animated: false)

        let transitionActions = {
            if let containerView = self.modalRootContainer.modalPresentationContainerView {
                self.parentRootContainer.addSettingsButtonToPresentationContainer(containerView)
            }
        }

        if actualStyle == .formSheet {
            // Add settings button into the modal container to make it accessible by users
            if let transitionCoordinator = transitionCoordinator {
                transitionCoordinator.animate { _ in
                    transitionActions()
                }
            } else {
                transitionActions()
            }
        } else {
            // Move settings button back into header bar
            finishPresentation()
        }
    }

    func presentationControllerDidDismiss(_ presentationController: UIPresentationController) {
        finishPresentation()
    }

    @objc private func dismissalTransitionDidEnd(_ notification: Notification) {
        guard let isCompleted = notification
            .userInfo?[
                UIPresentationController
                    .dismissalTransitionDidEndCompletedUserInfoKey
            ] as? NSNumber else { return }

        if isCompleted.boolValue {
            finishPresentation()
        }
    }
}

private extension UIViewController {
    /// Returns private `UITransitionView` used by UIKit that acts as a container view for modally
    /// presented controllers. When implementing a presentation controller subclass, this view
    /// is the one that is used to add additional decorations.
    var modalPresentationContainerView: UIView? {
        var currentView = view
        let iterator = AnyIterator { () -> UIView? in
            currentView = currentView?.superview
            return currentView
        }
        return iterator.first { view -> Bool in
            return view.description.starts(with: "<UITransitionView")
        }
    }
}
