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
        HeaderBarPresentation(style: .default, showsDivider: false)
    }
}

/// A protocol that defines the relationship between the root container and its child controllers
protocol RootContainment {
    /// Return the preferred header bar style
    var preferredHeaderBarPresentation: HeaderBarPresentation { get }

    /// Return true if the view controller prefers header bar hidden
    var prefersHeaderBarHidden: Bool { get }

    /// Return true if the view controller prefers notification bar hidden
    var prefersNotificationBarHidden: Bool { get }

    /// Return true if the view controller prefers device info bar hidden
    var prefersDeviceInfoBarHidden: Bool { get }
}

extension RootContainment {
    var prefersNotificationBarHidden: Bool {
        false
    }

    var prefersDeviceInfoBarHidden: Bool {
        false
    }
}

protocol RootContainerViewControllerDelegate: AnyObject {
    func rootContainerViewControllerShouldShowAccount(
        _ controller: RootContainerViewController,
        animated: Bool
    )

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
    let transitionContainer = UIView(frame: UIScreen.main.bounds)
    private var presentationContainerAccountButton: UIButton?
    private var presentationContainerSettingsButton: UIButton?
    private var configuration = RootConfiguration(showsAccountButton: false)

    private(set) var headerBarPresentation = HeaderBarPresentation.default
    private(set) var headerBarHidden = false
    private(set) var overrideHeaderBarHidden: Bool?

    private(set) var viewControllers = [UIViewController]()

    private var appearingController: UIViewController?
    private var disappearingController: UIViewController?
    private var interfaceOrientationMask: UIInterfaceOrientationMask?
    private var isNavigationBarHidden = false {
        didSet {
            guard let notificationController else {
                return
            }
            isNavigationBarHidden
                ? removeNotificationController(notificationController)
                : addNotificationController(notificationController)
        }
    }

    var topViewController: UIViewController? {
        viewControllers.last
    }

    weak var delegate: RootContainerViewControllerDelegate?

    override var childForStatusBarStyle: UIViewController? {
        topViewController
    }

    override var childForStatusBarHidden: UIViewController? {
        topViewController
    }

    override var shouldAutomaticallyForwardAppearanceMethods: Bool {
        false
    }

    override var disablesAutomaticKeyboardDismissal: Bool {
        topViewController?.disablesAutomaticKeyboardDismissal ?? super
            .disablesAutomaticKeyboardDismissal
    }

    // MARK: - View lifecycle

    override func viewDidLoad() {
        super.viewDidLoad()
        var margins = view.directionalLayoutMargins
        margins.leading = UIMetrics.contentLayoutMargins.leading
        margins.trailing = UIMetrics.contentLayoutMargins.trailing
        view.directionalLayoutMargins = margins

        definesPresentationContext = true

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
        true
    }

    override var supportedInterfaceOrientations: UIInterfaceOrientationMask {
        interfaceOrientationMask ?? super.supportedInterfaceOrientations
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

    func popToViewController(
        _ controller: UIViewController,
        animated: Bool,
        completion: CompletionHandler? = nil
    ) {
        guard let index = viewControllers.firstIndex(of: controller) else { return }

        let newViewControllers = Array(viewControllers[...index])

        setViewControllersInternal(
            newViewControllers,
            isUnwinding: true,
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
    func showAccount(animated: Bool) {
        delegate?.rootContainerViewControllerShouldShowAccount(
            self,
            animated: animated
        )
    }

    /// Request to display settings controller
    func showSettings(navigateTo route: SettingsNavigationRoute? = nil, animated: Bool) {
        delegate?.rootContainerViewControllerShouldShowSettings(
            self,
            navigateTo: route,
            animated: animated
        )
    }

    /// Add account and settings bar buttons into the presentation container to make them accessible even
    /// when the root container is covered with a modal.
    func addTrailingButtonsToPresentationContainer(_ presentationContainer: UIView) {
        let accountButton = getPresentationContainerAccountButton()
        let settingsButton = getPresentationContainerSettingsButton()

        presentationContainerAccountButton = accountButton
        presentationContainerSettingsButton = settingsButton

        // Hide the account button inside the header bar to avoid color blending issues
        headerBarView.accountButton.alpha = 0
        headerBarView.settingsButton.alpha = 0

        presentationContainer.addConstrainedSubviews([accountButton, settingsButton]) {
            accountButton.centerXAnchor
                .constraint(equalTo: headerBarView.accountButton.centerXAnchor)
            accountButton.centerYAnchor
                .constraint(equalTo: headerBarView.accountButton.centerYAnchor)

            settingsButton.centerXAnchor
                .constraint(equalTo: headerBarView.settingsButton.centerXAnchor)
            settingsButton.centerYAnchor
                .constraint(equalTo: headerBarView.settingsButton.centerYAnchor)
        }
    }

    func removeTrailingButtonsFromPresentationContainer() {
        presentationContainerAccountButton?.removeFromSuperview()
        presentationContainerSettingsButton?.removeFromSuperview()

        headerBarView.accountButton.alpha = 1
        headerBarView.settingsButton.alpha = 1
    }

    func setOverrideHeaderBarHidden(_ isHidden: Bool?, animated: Bool) {
        overrideHeaderBarHidden = isHidden

        if let isHidden {
            setHeaderBarHidden(isHidden, animated: animated)
        } else {
            updateHeaderBarHiddenFromChildPreferences(animated: animated)
        }
    }

    // MARK: - Accessibility

    override func accessibilityPerformMagicTap() -> Bool {
        delegate?.rootContainerViewAccessibilityPerformMagicTap(self) ?? super
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

        headerBarView.accountButton.addTarget(
            self,
            action: #selector(handleAccountButtonTap),
            for: .touchUpInside
        )

        headerBarView.settingsButton.addTarget(
            self,
            action: #selector(handleSettingsButtonTap),
            for: .touchUpInside
        )

        view.addSubview(headerBarView)

        NSLayoutConstraint.activate(constraints)
    }

    private func getPresentationContainerAccountButton() -> UIButton {
        let button: UIButton

        if let transitionViewButton = presentationContainerAccountButton {
            transitionViewButton.removeFromSuperview()
            button = transitionViewButton
        } else {
            button = HeaderBarView.makeHeaderBarButton(with: UIImage(named: "IconAccount"))
            button.addTarget(
                self,
                action: #selector(handleAccountButtonTap),
                for: .touchUpInside
            )
        }

        button.isEnabled = headerBarView.accountButton.isEnabled
        button.isHidden = !configuration.showsAccountButton

        return button
    }

    private func getPresentationContainerSettingsButton() -> UIButton {
        let button: UIButton

        if let transitionViewButton = presentationContainerSettingsButton {
            transitionViewButton.removeFromSuperview()
            button = transitionViewButton
        } else {
            button = HeaderBarView.makeHeaderBarButton(with: UIImage(named: "IconSettings"))
            button.isEnabled = headerBarView.settingsButton.isEnabled
            button.addTarget(
                self,
                action: #selector(handleSettingsButtonTap),
                for: .touchUpInside
            )
        }

        return button
    }

    @objc private func handleAccountButtonTap() {
        showAccount(animated: true)
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
        assert(
            Set(newViewControllers).count == newViewControllers.count,
            "All view controllers in root container controller must be distinct"
        )

        guard viewControllers != newViewControllers else {
            completion?()
            return
        }

        // Dot not handle appearance events when the container itself is not visible
        let shouldHandleAppearanceEvents = view.window != nil

        // Animations won't run when the container is not visible, so prevent them
        let shouldAnimate = animated && shouldHandleAppearanceEvents

        let sourceViewController = topViewController
        let targetViewController = newViewControllers.last

        let viewControllersToAdd = newViewControllers.filter { !viewControllers.contains($0) }
        let viewControllersToRemove = viewControllers.filter { !newViewControllers.contains($0) }

        // hide in-App notificationBanner when the container decides to keep it invisible
        isNavigationBarHidden = (targetViewController as? RootContainment)?.prefersNotificationBarHidden ?? false

        let finishTransition = {
            /*
             Finish transition appearance.
             Note this has to be done before the call to `didMove(to:)` or `removeFromParent()`
             otherwise `endAppearanceTransition()` will fire `didMove(to:)` twice.
             */
            if shouldHandleAppearanceEvents {
                if let targetViewController,
                   sourceViewController != targetViewController {
                    self.endChildControllerTransition(targetViewController)
                }

                if let sourceViewController,
                   sourceViewController != targetViewController {
                    self.endChildControllerTransition(sourceViewController)
                }
            }

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

            self.updateInterfaceOrientation(attemptRotateToDeviceOrientation: true)
            self.updateAccessibilityElementsAndNotifyScreenChange()

            completion?()
        }

        let alongSideAnimations = {
            self.updateHeaderBarStyleFromChildPreferences(animated: shouldAnimate)
            self.updateHeaderBarHiddenFromChildPreferences(animated: shouldAnimate)
            self.updateNotificationBarHiddenFromChildPreferences()
            self.updateDeviceInfoBarHiddenFromChildPreferences()
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
            if let sourceViewController,
               sourceViewController != targetViewController {
                beginChildControllerTransition(
                    sourceViewController,
                    isAppearing: false,
                    animated: shouldAnimate
                )
            }
            if let targetViewController,
               sourceViewController != targetViewController {
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

    private func updateDeviceInfoBarHiddenFromChildPreferences() {
        if let conforming = topViewController as? RootContainment {
            headerBarView.isDeviceInfoHidden = conforming.prefersDeviceInfoBarHidden
        }
    }

    private func updateNotificationBarHiddenFromChildPreferences() {
        if let notificationController,
           let conforming = topViewController as? RootContainment {
            conforming.prefersNotificationBarHidden
                ? removeNotificationController(notificationController)
                : addNotificationController(notificationController)
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

    // MARK: - Notification controller support

    /**
     An instance of notification controller presented within container.
     */
    var notificationController: NotificationController? {
        didSet {
            guard oldValue != notificationController else { return }

            oldValue.flatMap { removeNotificationController($0) }
            notificationController.flatMap { addNotificationController($0) }
        }
    }

    /**
     Layout guide for notification view.

     When set, notification view follows the layout guide that defines its dimensions and position, otherwise it's
     laid out within container's safe area.
     */
    var notificationViewLayoutGuide: UILayoutGuide? {
        didSet {
            notificationController.flatMap { updateNotificationViewConstraints($0) }
        }
    }

    private var notificationViewConstraints: [NSLayoutConstraint] = []

    private func updateNotificationViewConstraints(_ notificationController: NotificationController) {
        let newConstraints = notificationController.view
            .pinEdgesTo(notificationViewLayoutGuide ?? view.safeAreaLayoutGuide)

        NSLayoutConstraint.deactivate(notificationViewConstraints)
        NSLayoutConstraint.activate(newConstraints)

        notificationViewConstraints = newConstraints
    }

    private func addNotificationController(_ notificationController: NotificationController) {
        guard let notificationView = notificationController.view else { return }

        notificationView.configureForAutoLayout()

        addChild(notificationController)
        view.addSubview(notificationView)
        notificationController.didMove(toParent: self)

        updateNotificationViewConstraints(notificationController)
    }

    private func removeNotificationController(_ notificationController: NotificationController) {
        notificationController.willMove(toParent: nil)
        notificationController.view.removeFromSuperview()
        notificationController.removeFromParent()
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

extension RootContainerViewController {
    func update(configuration: RootConfiguration) {
        self.configuration = configuration
        presentationContainerAccountButton?.isHidden = !configuration.showsAccountButton
        headerBarView.update(configuration: configuration)
    }
}
