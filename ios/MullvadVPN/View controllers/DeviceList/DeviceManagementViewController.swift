//
//  DeviceManagementViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 15/07/2022.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

@preconcurrency import MullvadLogging
import MullvadREST
import MullvadTypes
import Operations
import UIKit

protocol DeviceManagementViewControllerDelegate: AnyObject, Sendable {
    func deviceManagementViewControllerDidFinish(_ controller: DeviceManagementViewController)
    func deviceManagementViewControllerDidCancel(_ controller: DeviceManagementViewController)
}

class DeviceManagementViewController: UIViewController, RootContainment {
    weak var delegate: DeviceManagementViewControllerDelegate?

    var preferredHeaderBarPresentation: HeaderBarPresentation {
        .default
    }

    var prefersHeaderBarHidden: Bool {
        false
    }

    override var preferredStatusBarStyle: UIStatusBarStyle {
        .lightContent
    }

    private let contentView: DeviceManagementContentView = {
        let contentView = DeviceManagementContentView()
        contentView.translatesAutoresizingMaskIntoConstraints = false
        return contentView
    }()

    nonisolated(unsafe) private let logger = Logger(label: "DeviceManagementViewController")
    let interactor: DeviceManagementInteractor
    private let alertPresenter: AlertPresenter

    init(interactor: DeviceManagementInteractor, alertPresenter: AlertPresenter) {
        self.interactor = interactor
        self.alertPresenter = alertPresenter

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
            Task { @MainActor in
                self?.handleDeviceDeletion(viewModel, completionHandler: finish)
            }
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
        completionHandler: (@Sendable (Result<Void, Error>) -> Void)? = nil
    ) {
        interactor.getDevices { [weak self] result in
            guard let self = self else { return }

            if let devices = result.value {
                setDevices(devices, animated: animateUpdates)
            }

            completionHandler?(result.map { _ in () })
        }
    }

    // MARK: - Private

    nonisolated private func setDevices(_ devices: [Device], animated: Bool) {
        let viewModels = devices.map { restDevice -> DeviceViewModel in
            DeviceViewModel(
                id: restDevice.id,
                name: restDevice.name.capitalized,
                creationDate: DateFormatter.localizedString(
                    from: restDevice.created,
                    dateStyle: .short,
                    timeStyle: .none
                )
            )
        }

        Task { @MainActor in
            contentView.canContinue = viewModels.count < ApplicationConfiguration.maxAllowedDevices
            contentView.setDeviceViewModels(viewModels, animated: animated)
        }
    }

    private func handleDeviceDeletion(
        _ device: DeviceViewModel,
        completionHandler: @escaping @Sendable () -> Void
    ) {
        showLogoutConfirmation(deviceName: device.name) { [weak self] shouldDelete in
            guard let self else { return }

            guard shouldDelete else {
                completionHandler()
                return
            }

            deleteDevice(identifier: device.id) { [weak self] error in
                guard let self = self else { return }

                if let error {
                    Task { @MainActor in
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
        let presentation = AlertPresentation(
            id: "delete-device-error-alert",
            title: title,
            message: getErrorDescription(error),
            buttons: [
                AlertAction(
                    title: NSLocalizedString(
                        "ERROR_ALERT_OK_ACTION",
                        tableName: "DeviceManagement",
                        value: "Got it!",
                        comment: ""
                    ),
                    style: .default
                ),
            ]
        )

        alertPresenter.showAlert(presentation: presentation, animated: true)
    }

    private func showLogoutConfirmation(
        deviceName: String,
        completion: @escaping (_ shouldDelete: Bool) -> Void
    ) {
        let text = String(
            format: NSLocalizedString(
                "DELETE_ALERT_TITLE",
                tableName: "DeviceManagement",
                value: "Are you sure you want to log **\(deviceName)** out?",
                comment: ""
            )
        )

        let attributedText = NSAttributedString(
            markdownString: text,
            options: MarkdownStylingOptions(
                font: .preferredFont(forTextStyle: .body)
            )
        )

        let presentation = AlertPresentation(
            id: "logout-confirmation-alert",
            icon: .alert,
            attributedMessage: attributedText,
            buttons: [
                AlertAction(
                    title: NSLocalizedString(
                        "DELETE_ALERT_CONFIRM_ACTION",
                        tableName: "DeviceManagement",
                        value: "Yes, log out device",
                        comment: ""
                    ),
                    style: .destructive,
                    accessibilityId: .logOutDeviceConfirmButton,
                    handler: {
                        completion(true)
                    }
                ),
                AlertAction(
                    title: NSLocalizedString(
                        "DELETE_ALERT_CANCEL_ACTION",
                        tableName: "DeviceManagement",
                        value: "Back",
                        comment: ""
                    ),
                    style: .default,
                    accessibilityId: .logOutDeviceCancelButton,
                    handler: {
                        completion(false)
                    }
                ),
            ]
        )

        alertPresenter.showAlert(presentation: presentation, animated: true)
    }

    private func deleteDevice(identifier: String, completionHandler: @escaping @Sendable (Error?) -> Void) {
        interactor.deleteDevice(identifier) { [weak self] completion in
            guard let self = self else { return }

            switch completion {
            case .success:
                Task { @MainActor in
                    fetchDevices(animateUpdates: true) { completion in
                        completionHandler(completion.error)
                    }
                }

            case let .failure(error):
                if error.isOperationCancellationError {
                    completionHandler(nil)
                } else {
                    logger.error(
                        error: error,
                        message: "Failed to delete device."
                    )
                    completionHandler(error)
                }
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

struct DeviceViewModel: Sendable {
    let id: String
    let name: String
    let creationDate: String
}
