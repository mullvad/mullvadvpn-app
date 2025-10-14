//
//  EditCustomListCoordinator.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-02-15.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Combine
import MullvadSettings
import Routing
import UIKit

class EditCustomListCoordinator: Coordinator, Presentable, Presenting {
    enum FinishAction {
        case save, delete
    }

    let navigationController: UINavigationController
    let customListInteractor: CustomListInteractorProtocol
    let customList: CustomList
    let nodes: [LocationNode]
    let subject: CurrentValueSubject<CustomListViewModel, Never>
    private lazy var alertPresenter: AlertPresenter = {
        AlertPresenter(context: self)
    }()

    var presentedViewController: UIViewController {
        navigationController
    }

    var didFinish: ((EditCustomListCoordinator, FinishAction, CustomList) -> Void)?
    var didCancel: ((EditCustomListCoordinator) -> Void)?

    init(
        navigationController: UINavigationController,
        customListInteractor: CustomListInteractorProtocol,
        customList: CustomList,
        nodes: [LocationNode]
    ) {
        self.navigationController = navigationController
        self.customListInteractor = customListInteractor
        self.customList = customList
        self.nodes = nodes
        self.subject = CurrentValueSubject(
            CustomListViewModel(
                id: customList.id,
                name: customList.name,
                locations: customList.locations,
                tableSections: [.name, .editLocations, .deleteList]
            ))
    }

    func start() {
        let controller = CustomListViewController(
            interactor: customListInteractor,
            subject: subject,
            alertPresenter: alertPresenter
        )
        controller.delegate = self

        controller.navigationItem.title = subject.value.name

        navigationController.interactivePopGestureRecognizer?.delegate = self
        navigationController.pushViewController(controller, animated: true)

        guard let interceptibleNavigationController = navigationController as? InterceptibleNavigationController else {
            return
        }

        interceptibleNavigationController.shouldPopViewController = { [weak self] viewController in
            guard
                let self,
                let customListViewController = viewController as? CustomListViewController,
                customListViewController.hasUnsavedChanges
            else { return true }

            presentUnsavedChangesDialog()
            return false
        }

        interceptibleNavigationController.shouldPopToViewController = { [weak self] viewController in
            guard
                let self,
                let customListViewController = viewController as? CustomListViewController,
                customListViewController.hasUnsavedChanges
            else { return true }

            presentUnsavedChangesDialog()
            return false
        }
    }

    private func presentUnsavedChangesDialog() {
        let message = NSMutableAttributedString(
            markdownString: NSLocalizedString("You have unsaved changes.", comment: ""),
            options: MarkdownStylingOptions(font: .preferredFont(forTextStyle: .body))
        )

        let presentation = AlertPresentation(
            id: "api-custom-lists-unsaved-changes-alert",
            icon: .alert,
            attributedMessage: message,
            buttons: [
                AlertAction(
                    title: NSLocalizedString("Discard changes", comment: ""),
                    style: .destructive,
                    handler: {
                        self.didCancel?(self)
                    }
                ),
                AlertAction(
                    title: NSLocalizedString("Cancel", comment: ""),
                    style: .default
                ),
            ]
        )

        alertPresenter.showAlert(presentation: presentation, animated: true)
    }
}

extension EditCustomListCoordinator: @preconcurrency CustomListViewControllerDelegate {
    func customListDidSave(_ list: CustomList) {
        didFinish?(self, .save, list)
    }

    func customListDidDelete(_ list: CustomList) {
        didFinish?(self, .delete, list)
    }

    func showLocations(_ list: CustomList) {
        let coordinator = EditLocationsCoordinator(
            navigationController: navigationController,
            nodes: nodes,
            subject: subject
        )

        coordinator.didFinish = { locationsCoordinator in
            Task { @MainActor in
                locationsCoordinator.removeFromParent()
            }
        }

        coordinator.start()

        addChild(coordinator)
    }
}

extension EditCustomListCoordinator: UIGestureRecognizerDelegate {
    // For some reason, intercepting `popViewController(animated: Bool)` in `InterceptibleNavigationController`
    // by SWIPING back leads to weird behaviour where subsequent navigation seem to happen systemwise but not
    // UI-wise. This leads to the UI freezing up, and the only remedy is to restart the app.
    //
    // To get around this issue we can intercept the back swipe gesture and manually perform the transition
    // instead, thereby bypassing the inner mechanisms that seem to go out of sync.
    func gestureRecognizerShouldBegin(_ gestureRecognizer: UIGestureRecognizer) -> Bool {
        guard gestureRecognizer == navigationController.interactivePopGestureRecognizer else {
            return true
        }
        navigationController.popViewController(animated: true)
        return false
    }
}
