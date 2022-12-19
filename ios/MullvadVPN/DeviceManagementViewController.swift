//
//  DeviceManagementViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 15/07/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import MullvadLogging
import MullvadREST
import Operations
import UIKit

protocol DeviceManagementViewControllerDelegate: AnyObject {
    func deviceManagementViewControllerDidFinish(_ controller: DeviceManagementViewController)
    func deviceManagementViewControllerDidCancel(_ controller: DeviceManagementViewController)
}

class DeviceManagementViewController: UIViewController, RootContainment {
    weak var delegate: DeviceManagementViewControllerDelegate?

    var preferredHeaderBarPresentation: HeaderBarPresentation {
        return .default
    }

    var prefersHeaderBarHidden: Bool {
        return false
    }

    override var preferredStatusBarStyle: UIStatusBarStyle {
        return .lightContent
    }

    private let alertPresenter = AlertPresenter()

    private let contentView: DeviceManagementContentView = {
        let contentView = DeviceManagementContentView()
        contentView.translatesAutoresizingMaskIntoConstraints = false
        return contentView
    }()

    private let logger = Logger(label: "DeviceManagementViewController")
    private let interactor: DeviceManagementInteractor

    init(interactor: DeviceManagementInteractor) {
        self.interactor = interactor

        super.init(nibName: nil, bundle: nil)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func viewDidLoad() {
        super.viewDidLoad()

        view.backgroundColor = .secondaryColor

        view.addSubview(contentView)

        contentView.backButton.addTarget(
            self,
            action: #selector(didTapBackButton(_:)),
            for: .touchUpInside
        )

        contentView.continueButton.addTarget(
            self,
            action: #selector(didTapContinueButton(_:)),
            for: .touchUpInside
        )

        contentView.handleDeviceDeletion = { [weak self] viewModel, finish in
            self?.handleDeviceDeletion(viewModel, completionHandler: finish)
        }

        NSLayoutConstraint.activate([
            contentView.topAnchor.constraint(equalTo: view.safeAreaLayoutGuide.topAnchor),
            contentView.leadingAnchor.constraint(equalTo: view.leadingAnchor),
            contentView.trailingAnchor.constraint(equalTo: view.trailingAnchor),
            contentView.bottomAnchor.constraint(equalTo: view.bottomAnchor),
        ])
    }

    func fetchDevices(
        animateUpdates: Bool,
        completionHandler: ((OperationCompletion<Void, Error>) -> Void)? = nil
    ) {
        interactor.getDevices { [weak self] completion in
            guard let self = self else { return }

            if let devices = completion.value {
                self.setDevices(devices, animated: animateUpdates)
            }

            completionHandler?(completion.ignoreOutput())
        }
    }

    // MARK: - Private

    private func setDevices(_ devices: [REST.Device], animated: Bool) {
        let viewModels = devices.map { restDevice -> DeviceViewModel in
            return DeviceViewModel(
                id: restDevice.id,
                name: restDevice.name
            )
        }

        contentView.canContinue = viewModels.count < ApplicationConfiguration.maxAllowedDevices
        contentView.setDeviceViewModels(viewModels, animated: animated)
    }

    private func handleDeviceDeletion(
        _ device: DeviceViewModel,
        completionHandler: @escaping () -> Void
    ) {
        showDeleteConfirmation(deviceName: device.displayName) { [weak self] shouldDelete in
            guard let self = self else { return }

            guard shouldDelete else {
                completionHandler()
                return
            }

            self.deleteDevice(identifier: device.id) { error in
                if let error = error {
                    self.showErrorAlert(
                        title: NSLocalizedString(
                            "LOGOUT_DEVICE_ERROR_ALERT_TITLE",
                            tableName: "DeviceManagement",
                            value: "Failed to log out device",
                            comment: ""
                        ),
                        error: error
                    )
                }

                completionHandler()
            }
        }
    }

    private func getErrorDescription(_ error: Error) -> String {
        if case let .network(urlError) = error as? REST.Error {
            return urlError.localizedDescription
        } else {
            return error.localizedDescription
        }
    }

    private func showErrorAlert(title: String, error: Error) {
        let alertController = UIAlertController(
            title: title,
            message: getErrorDescription(error),
            preferredStyle: .alert
        )

        alertController.addAction(
            UIAlertAction(
                title: NSLocalizedString(
                    "ERROR_ALERT_OK_ACTION",
                    tableName: "DeviceManagement",
                    value: "OK",
                    comment: ""
                ),
                style: .cancel
            )
        )

        alertPresenter.enqueue(alertController, presentingController: self)
    }

    private func showDeleteConfirmation(
        deviceName: String,
        completion: @escaping (_ shouldDelete: Bool) -> Void
    ) {
        let localizedTitle = String(
            format: NSLocalizedString(
                "DELETE_ALERT_TITLE",
                tableName: "DeviceManagement",
                value: "Are you sure you want to log %@ out?",
                comment: ""
            ), deviceName
        )

        let alertController = UIAlertController(
            title: localizedTitle,
            message: nil,
            preferredStyle: .alert
        )

        let actions = [
            UIAlertAction(
                title: NSLocalizedString(
                    "DELETE_ALERT_CANCEL_ACTION",
                    tableName: "DeviceManagement",
                    value: "Back",
                    comment: ""
                ),
                style: .cancel,
                handler: { _ in
                    completion(false)
                }
            ),
            UIAlertAction(
                title: NSLocalizedString(
                    "DELETE_ALERT_CONFIRM_ACTION",
                    tableName: "DeviceManagement",
                    value: "Yes, log out device",
                    comment: ""
                ),
                style: .destructive,
                handler: { _ in
                    completion(true)
                }
            ),
        ]

        for action in actions {
            alertController.addAction(action)
        }

        alertPresenter.enqueue(alertController, presentingController: self)
    }

    private func deleteDevice(identifier: String, completionHandler: @escaping (Error?) -> Void) {
        interactor.deleteDevice(identifier) { [weak self] completion in
            guard let self = self else { return }

            switch completion {
            case .success:
                self.fetchDevices(animateUpdates: true) { completion in
                    completionHandler(completion.error)
                }

            case let .failure(error):
                self.logger.error(
                    error: error,
                    message: "Failed to delete device."
                )
                completionHandler(error)

            case .cancelled:
                completionHandler(nil)
            }
        }
    }

    // MARK: - Actions

    @objc private func didTapBackButton(_ sender: Any?) {
        delegate?.deviceManagementViewControllerDidCancel(self)
    }

    @objc private func didTapContinueButton(_ sender: Any?) {
        delegate?.deviceManagementViewControllerDidFinish(self)
    }
}

struct DeviceViewModel {
    var id: String
    var name: String

    var displayName: String {
        return name.capitalized
    }
}
