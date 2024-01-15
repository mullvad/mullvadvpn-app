//
//  IPOverrideCoordinator.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-01-15.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import MullvadTypes
import Routing
import UIKit

class IPOverrideCoordinator: Coordinator, Presenting, SettingsChildCoordinator {
    let navigationController: UINavigationController
    let repository: IPOverrideRepositoryProtocol

    lazy var ipOverrideViewController: IPOverrideViewController = {
        let viewController = IPOverrideViewController(alertPresenter: AlertPresenter(context: self))
        viewController.delegate = self
        return viewController
    }()

    var presentationContext: UIViewController {
        navigationController
    }

    init(navigationController: UINavigationController, repository: IPOverrideRepositoryProtocol) {
        self.navigationController = navigationController
        self.repository = repository
    }

    func start(animated: Bool) {
        navigationController.pushViewController(ipOverrideViewController, animated: animated)
        resetToDefaultStatus()
    }

    private func showImportTextView() {
        let viewController = IPOverrideTextViewController()
        let customNavigationController = CustomNavigationController(rootViewController: viewController)

        viewController.didFinishEditing = { [weak self] text in
            if let data = text.data(using: .utf8) {
                self?.handleImport(of: data, context: .text)
            } else {
                self?.ipOverrideViewController.setStatus(.importFailed(.text))
                print("Could not convert string to data: \(text)")
            }
        }

        presentationContext.present(customNavigationController, animated: true)
    }

    private func showImportFileView() {
        let documentPicker = UIDocumentPickerViewController(forOpeningContentTypes: [.json, .text])
        documentPicker.delegate = self

        presentationContext.present(documentPicker, animated: true)
    }

    private func resetToDefaultStatus(delay: Duration = .zero) {
        DispatchQueue.main.asyncAfter(deadline: .now() + delay.timeInterval) { [weak self] in
            if self?.repository.fetchAll().isEmpty == false {
                self?.ipOverrideViewController.setStatus(.active)
            } else {
                self?.ipOverrideViewController.setStatus(.noImports)
            }
        }
    }

    private func handleImport(of data: Data, context: IPOverrideStatus.Context) {
        do {
            let overrides = try repository.parseData(data)

            repository.add(overrides)
            ipOverrideViewController.setStatus(.importSuccessful(context))
        } catch {
            ipOverrideViewController.setStatus(.importFailed(context))
            print("Error importing ip overrides: \(error)")
        }

        resetToDefaultStatus(delay: .seconds(10))
    }
}

extension IPOverrideCoordinator: IPOverrideViewControllerDelegate {
    func controllerShouldShowTextImportView(_ controller: IPOverrideViewController) {
        showImportTextView()
    }

    func controllerShouldShowFileImportView(_ controller: IPOverrideViewController) {
        showImportFileView()
    }

    func controllerShouldClearAllOverrides(_ controller: IPOverrideViewController) {
        repository.deleteAll()
        resetToDefaultStatus()
    }
}

extension IPOverrideCoordinator: UIDocumentPickerDelegate {
    func documentPicker(_ controller: UIDocumentPickerViewController, didPickDocumentsAt urls: [URL]) {
        if let url = urls.first {
            do {
                let data = try Data(contentsOf: url)
                handleImport(of: data, context: .file)
            } catch {
                ipOverrideViewController.setStatus(.importFailed(.file))
                print("Could not convert file at url to data: \(url)")
            }
        }
    }
}
