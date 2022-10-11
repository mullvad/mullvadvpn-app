//
//  RootContainerViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 25/05/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import UIKit

enum HeaderBarStyle {
    case transparent, `default`, unsecured, secured

    fileprivate func backgroundColor() -> UIColor {
        switch self {
        case .transparent:
            return UIColor.clear
        case .default:
            return UIColor.HeaderBar.defaultBackgroundColor
        case .secured:
            return UIColor.HeaderBar.securedBackgroundColor
        case .unsecured:
            return UIColor.HeaderBar.unsecuredBackgroundColor
        }
    }
}

struct HeaderBarPresentation {
    let style: HeaderBarStyle
    let showsDivider: Bool

    static var `default`: HeaderBarPresentation {
        return HeaderBarPresentation(style: .default, showsDivider: false)
    }
}

/// A protocol that defines the relationship between the root container and its child controllers
protocol RootContainment {
    /// Return the preferred header bar style
    var preferredHeaderBarPresentation: HeaderBarPresentation { get }

    /// Return true if the view controller prefers header bar hidden
    var prefersHeaderBarHidden: Bool { get }
}

protocol RootContainerViewControllerDelegate: AnyObject {
    func rootContainerViewControllerShouldShowSettings(
        _ controller: RootContainerViewController,
        navigateTo route: SettingsNavigationRoute?,
        animated: Bool
    )

    func rootContainerViewSupportedInterfaceOrientations(_ controller: RootContainerViewController)
        -> UIInterfaceOrientationMask

    func rootContainerViewAccessibilityPerformMagicTap(_ controller: RootContainerViewController)
        -> Bool
}

/// A root container view controller
class RootContainerViewController: UIViewController {
    typealias CompletionHandler = () -> Void

    private let headerBarView = HeaderBarView(frame: CGRect(x: 0, y: 0, width: 100, height: 100))
    private let transitionContainer = UIView(frame: UIScreen.main.bounds)
    private var presentationContainerSettingsButton: UIButton?

    private(set) var headerBarPresentation = HeaderBarPresentation.default
    private(set) var headerBarHidden = false
    private(set) var overrideHeaderBarHidden: Bool?

    private(set) var viewControllers = [UIViewController]()

    private var appearingController: UIViewController?
    private var disappearingController: UIViewController?
    private var interfaceOrientationMask: UIInterfaceOrientationMask?

    var topViewController: UIViewController? {
        return viewControllers.last
    }

    weak var delegate: RootContainerViewControllerDelegate?

    override var childForStatusBarStyle: UIViewController? {
        return topViewController
    }

    override var childForStatusBarHidden: UIViewController? {
        return topViewController
    }

    override var shouldAutomaticallyForwardAppearanceMethods: Bool {
        return false
    }

    override var disablesAutomaticKeyboardDismissal: Bool {
        return topViewController?.disablesAutomaticKeyboardDismissal ?? super
            .disablesAutomaticKeyboardDismissal
    }

    // MARK: - View lifecycle

    override func viewDidLoad() {
        super.viewDidLoad()

        var margins = view.layoutMargins
        margins.left = 24
        margins.right = 24
        view.layoutMargins = margins

        addTransitionView()
        addHeaderBarView()
        updateHeaderBarBackground()
    }

    override func viewDidLayoutSubviews() {
        super.viewDidLayoutSubviews()

        updateAdditionalSafeAreaInsetsIfNeeded()
    }

    override func viewSafeAreaInsetsDidChange() {
        super.viewSafeAreaInsetsDidChange()

        updateHeaderBarLayoutMarginsIfNeeded()
    }

    override func viewWillAppear(_ animated: Bool) {
        super.viewWillAppear(animated)

        if let childController = topViewController {
            beginChildControllerTransition(childController, isAppearing: true, animated: animated)
        }
    }

    override func viewDidAppear(_ animated: Bool) {
        super.viewDidAppear(animated)

        if let childController = topViewController {
            endChildControllerTransition(childController)
        }
    }

    override func viewWillDisappear(_ animated: Bool) {
        super.viewWillDisappear(animated)

        if let childController = topViewController {
            beginChildControllerTransition(childController, isAppearing: false, animated: animated)
        }
    }

    override func viewDidDisappear(_ animated: Bool) {
        super.viewDidDisappear(animated)

        if let childController = topViewController {
            endChildControllerTransition(childController)
        }
    }

    // MARK: - Autorotation

    override var shouldAutorotate: Bool {
        return true
    }

    override var supportedInterfaceOrientations: UIInterfaceOrientationMask {
        return interfaceOrientationMask ?? super.supportedInterfaceOrientations
    }

    // MARK: - Public

    func setViewControllers(
        _ newViewControllers: [UIViewController],
        animated: Bool,
        completion: CompletionHandler? = nil
    ) {
        // Fetch the initial orientation mask
        if interfaceOrientationMask == nil {
            updateInterfaceOrientation(attemptRotateToDeviceOrientation: false)
        }

        setViewControllersInternal(
            newViewControllers,
            isUnwinding: false,
            animated: animated,
            completion: completion
        )
    }

    func pushViewController(
        _ viewController: UIViewController,
        animated: Bool,
        completion: CompletionHandler? = nil
    ) {
        var newViewControllers = viewControllers.filter { $0 != viewController }
        newViewControllers.append(viewController)

        setViewControllersInternal(
            newViewControllers,
            isUnwinding: false,
            animated: animated,
            completion: completion
        )
    }

    func popViewController(animated: Bool, completion: CompletionHandler? = nil) {
        guard viewControllers.count > 1 else { return }

        var newViewControllers = viewControllers
        newViewControllers.removeLast()

        setViewControllersInternal(
            newViewControllers,
            isUnwinding: true,
            animated: animated,
            completion: completion
        )
    }

    func popToRootViewController(animated: Bool, completion: CompletionHandler? = nil) {
        if let rootController = viewControllers.first, viewControllers.count > 1 {
            setViewControllersInternal(
                [rootController],
                isUnwinding: true,
                animated: animated,
                completion: completion
            )
        }
    }

    /// Request the root container to query the top controller for the new header bar style
    func updateHeaderBarAppearance() {
        updateHeaderBarStyleFromChildPreferences(animated: UIView.areAnimationsEnabled)
    }

    func updateHeaderBarHiddenAppearance() {
        updateHeaderBarHiddenFromChildPreferences(animated: UIView.areAnimationsEnabled)
    }

    /// Request to display settings controller
    func showSettings(navigateTo route: SettingsNavigationRoute? = nil, animated: Bool) {
        delegate?.rootContainerViewControllerShouldShowSettings(
            self,
            navigateTo: route,
            animated: animated
        )
    }

    /// Enable or disable the settings bar button displayed in the header bar
    func setEnableSettingsButton(_ isEnabled: Bool) {
        headerBarView.settingsButton.isEnabled = isEnabled
        presentationContainerSettingsButton?.isEnabled = isEnabled
    }

    /// Add settings bar button into the presentation container to make settings accessible even
    /// when the root container is covered with modal.
    func addSettingsButtonToPresentationContainer(_ presentationContainer: UIView) {
        let settingsButton: UIButton

        if let transitionViewSettingsButton = presentationContainerSettingsButton {
            transitionViewSettingsButton.removeFromSuperview()
            settingsButton = transitionViewSettingsButton
        } else {
            settingsButton = HeaderBarView.makeSettingsButton()
            settingsButton.isEnabled = headerBarView.settingsButton.isEnabled
            settingsButton.addTarget(
                self,
                action: #selector(handleSettingsButtonTap),
                for: .touchUpInside
            )

            presentationContainerSettingsButton = settingsButton
        }

        // Hide the settings button inside the header bar to avoid color blending issues
        headerBarView.settingsButton.alpha = 0

        presentationContainer.addSubview(settingsButton)

        NSLayoutConstraint.activate([
            settingsButton.centerXAnchor
                .constraint(equalTo: headerBarView.settingsButton.centerXAnchor),
            settingsButton.centerYAnchor
                .constraint(equalTo: headerBarView.settingsButton.centerYAnchor),
        ])
    }

    func removeSettingsButtonFromPresentationContainer() {
        presentationContainerSettingsButton?.removeFromSuperview()
        headerBarView.settingsButton.alpha = 1
    }

    func setOverrideHeaderBarHidden(_ isHidden: Bool?, animated: Bool) {
        overrideHeaderBarHidden = isHidden

        if let isHidden = isHidden {
            setHeaderBarHidden(isHidden, animated: animated)
        } else {
            updateHeaderBarHiddenFromChildPreferences(animated: animated)
        }
    }

    // MARK: - Accessibility

    override func accessibilityPerformMagicTap() -> Bool {
        return delegate?.rootContainerViewAccessibilityPerformMagicTap(self) ?? super
            .accessibilityPerformMagicTap()
    }

    // MARK: - Private

    private func addTransitionView() {
        let constraints = [
            transitionContainer.topAnchor.constraint(equalTo: view.topAnchor),
            transitionContainer.leadingAnchor.constraint(equalTo: view.leadingAnchor),
            transitionContainer.trailingAnchor.constraint(equalTo: view.trailingAnchor),
            transitionContainer.bottomAnchor.constraint(equalTo: view.bottomAnchor),
        ]

        transitionContainer.translatesAutoresizingMaskIntoConstraints = false
        view.addSubview(transitionContainer)

        NSLayoutConstraint.activate(constraints)
    }

    private func addHeaderBarView() {
        let constraints = [
            headerBarView.topAnchor.constraint(equalTo: view.topAnchor),
            headerBarView.leadingAnchor.constraint(equalTo: view.leadingAnchor),
            headerBarView.trailingAnchor.constraint(equalTo: view.trailingAnchor),
        ]

        headerBarView.translatesAutoresizingMaskIntoConstraints = false

        // Prevent automatic layout margins adjustment as we manually control them.
        headerBarView.insetsLayoutMarginsFromSafeArea = false

        headerBarView.settingsButton.addTarget(
            self,
            action: #selector(handleSettingsButtonTap),
            for: .touchUpInside
        )

        view.addSubview(headerBarView)

        NSLayoutConstraint.activate(constraints)
    }

    @objc private func handleSettingsButtonTap() {
        showSettings(animated: true)
    }

    private func setViewControllersInternal(
        _ newViewControllers: [UIViewController],
        isUnwinding: Bool,
        animated: Bool,
        completion: CompletionHandler? = nil
    ) {
        // Dot not handle appearance events when the container itself is not visible
        let shouldHandleAppearanceEvents = view.window != nil

        // Animations won't run when the container is not visible, so prevent them
        let shouldAnimate = animated && shouldHandleAppearanceEvents

        let sourceViewController = topViewController
        let targetViewController = newViewControllers.last

        let viewControllersToAdd = newViewControllers.filter { !viewControllers.contains($0) }
        let viewControllersToRemove = viewControllers.filter { !newViewControllers.contains($0) }

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

            // Remove the source controller from view hierarchy
            if sourceViewController != targetViewController {
                sourceViewController?.view.removeFromSuperview()
            }

            // Finish appearance transition
            if shouldHandleAppearanceEvents {
                if let sourceViewController = sourceViewController {
                    self.endChildControllerTransition(sourceViewController)
                }

                if let targetViewController = targetViewController,
                   sourceViewController != targetViewController
                {
                    self.endChildControllerTransition(targetViewController)
                }
            }

            self.updateInterfaceOrientation(attemptRotateToDeviceOrientation: true)
            self.updateAccessibilityElementsAndNotifyScreenChange()

            completion?()
        }

        let alongSideAnimations = {
            self.updateHeaderBarStyleFromChildPreferences(animated: shouldAnimate)
            self.updateHeaderBarHiddenFromChildPreferences(animated: shouldAnimate)
        }

        // Add new child controllers. The call to addChild() automatically calls child.willMove()
        // Children have to be registered in the container for Storyboard unwind segues to function
        // properly, however the child controller views don't have to be added immediately, and
        // appearance methods have to be handled manually.
        for child in viewControllersToAdd {
            addChild(child)
        }

        // Make sure that all new view controllers have loaded their views
        // This is important because the unwind segue calls the unwind action which may rely on
        // IB outlets to be set at that time.
        for newViewController in newViewControllers {
            newViewController.loadViewIfNeeded()
        }

        // Add the destination view into the view hierarchy
        if let targetView = targetViewController?.view {
            addChildView(targetView)
        }

        // Notify the controllers that they will transition out of the container
        for child in viewControllersToRemove {
            child.willMove(toParent: nil)
        }

        viewControllers = newViewControllers

        // Begin appearance transition
        if shouldHandleAppearanceEvents {
            if let sourceViewController = sourceViewController {
                beginChildControllerTransition(
                    sourceViewController,
                    isAppearing: false,
                    animated: shouldAnimate
                )
            }
            if let targetViewController = targetViewController,
               sourceViewController != targetViewController
            {
                beginChildControllerTransition(
                    targetViewController,
                    isAppearing: true,
                    animated: shouldAnimate
                )
            }
            setNeedsStatusBarAppearanceUpdate()
        }

        if shouldAnimate {
            CATransaction.begin()
            CATransaction.setCompletionBlock {
                finishTransition()
            }

            let transition = CATransition()
            transition.duration = 0.35
            transition.type = .push

            // Pick the animation movement direction
            let sourceIndex = sourceViewController.flatMap { newViewControllers.firstIndex(of: $0) }
            let targetIndex = targetViewController.flatMap { newViewControllers.firstIndex(of: $0) }

            switch (sourceIndex, targetIndex) {
            case let (.some(lhs), .some(rhs)):
                transition.subtype = lhs > rhs ? .fromLeft : .fromRight
            case (.none, .some):
                transition.subtype = isUnwinding ? .fromLeft : .fromRight
            default:
                transition.subtype = .fromRight
            }

            transitionContainer.layer.add(transition, forKey: "transition")
            alongSideAnimations()

            CATransaction.commit()
        } else {
            alongSideAnimations()
            finishTransition()
        }
    }

    private func addChildView(_ childView: UIView) {
        childView.translatesAutoresizingMaskIntoConstraints = true
        childView.autoresizingMask = [.flexibleWidth, .flexibleHeight]
        childView.frame = transitionContainer.bounds

        transitionContainer.addSubview(childView)
    }

    /// Updates the header bar's layout margins to make sure it doesn't go below the system status
    /// bar.
    private func updateHeaderBarLayoutMarginsIfNeeded() {
        let offsetTop = view.safeAreaInsets.top - additionalSafeAreaInsets.top

        if headerBarView.layoutMargins.top != offsetTop {
            headerBarView.layoutMargins.top = offsetTop
        }
    }

    /// Updates additional safe area insets to push the child views below the header bar
    private func updateAdditionalSafeAreaInsetsIfNeeded() {
        let offsetTop = view.safeAreaInsets.top - additionalSafeAreaInsets.top
        let insetTop = headerBarHidden ? 0 : headerBarView.frame.height - offsetTop

        if additionalSafeAreaInsets.top != insetTop {
            additionalSafeAreaInsets.top = insetTop
        }
    }

    private func setHeaderBarPresentation(_ presentation: HeaderBarPresentation, animated: Bool) {
        headerBarPresentation = presentation

        let action = {
            self.updateHeaderBarBackground()
        }

        if animated {
            UIView.animate(withDuration: 0.25, animations: action)
        } else {
            action()
        }
    }

    private func setHeaderBarHidden(_ hidden: Bool, animated: Bool) {
        headerBarHidden = hidden

        let action = {
            self.headerBarView.alpha = hidden ? 0 : 1
        }

        if animated {
            UIView.animate(withDuration: 0.25, animations: action)
        } else {
            action()
        }
    }

    private func updateHeaderBarBackground() {
        headerBarView.backgroundColor = headerBarPresentation.style.backgroundColor()
        headerBarView.showsDivider = headerBarPresentation.showsDivider
    }

    private func updateHeaderBarStyleFromChildPreferences(animated: Bool) {
        if let conforming = topViewController as? RootContainment {
            setHeaderBarPresentation(conforming.preferredHeaderBarPresentation, animated: animated)
        }
    }

    private func updateHeaderBarHiddenFromChildPreferences(animated: Bool) {
        guard overrideHeaderBarHidden == nil else { return }

        if let conforming = topViewController as? RootContainment {
            setHeaderBarHidden(conforming.prefersHeaderBarHidden, animated: animated)
        }
    }

    private func updateInterfaceOrientation(attemptRotateToDeviceOrientation: Bool) {
        let newSupportedOrientations = delegate?
            .rootContainerViewSupportedInterfaceOrientations(self)

        if interfaceOrientationMask != newSupportedOrientations {
            interfaceOrientationMask = newSupportedOrientations

            // Tell UIKit to update the interface orientation
            if attemptRotateToDeviceOrientation {
                Self.attemptRotationToDeviceOrientation()
            }
        }
    }

    private func beginChildControllerTransition(
        _ controller: UIViewController,
        isAppearing: Bool,
        animated: Bool
    ) {
        if appearingController != controller, isAppearing {
            appearingController = controller
            controller.beginAppearanceTransition(isAppearing, animated: animated)
        }

        if disappearingController != controller, !isAppearing {
            disappearingController = controller
            controller.beginAppearanceTransition(isAppearing, animated: animated)
        }
    }

    private func endChildControllerTransition(_ controller: UIViewController) {
        if controller == appearingController {
            appearingController = nil
            controller.endAppearanceTransition()
        }

        if controller == disappearingController {
            disappearingController = nil
            controller.endAppearanceTransition()
        }
    }

    private func updateAccessibilityElementsAndNotifyScreenChange() {
        // Update accessibility elements to define the correct navigation order: header bar, content
        // view.
        view.accessibilityElements = [headerBarView, topViewController?.view].compactMap { $0 }

        // Tell accessibility that the significant part of screen was changed.
        UIAccessibility.post(notification: .screenChanged, argument: nil)
    }
}

/// A UIViewController extension that gives view controllers an access to root container
extension UIViewController {
    var rootContainerController: RootContainerViewController? {
        var current: UIViewController? = self
        let iterator = AnyIterator { () -> UIViewController? in
            current = current?.parent
            return current
        }

        return iterator.first { $0 is RootContainerViewController } as? RootContainerViewController
    }

    func setNeedsHeaderBarStyleAppearanceUpdate() {
        rootContainerController?.updateHeaderBarAppearance()
    }

    func setNeedsHeaderBarHiddenAppearanceUpdate() {
        rootContainerController?.updateHeaderBarHiddenAppearance()
    }
}
