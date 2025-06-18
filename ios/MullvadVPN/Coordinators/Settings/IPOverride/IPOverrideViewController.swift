//
//  IPOverrideViewController.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-01-15.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Combine
import MullvadSettings
import UIKit

class IPOverrideViewController: UIViewController {
    private let interactor: IPOverrideInteractor
    private var cancellables = Set<AnyCancellable>()
    private let alertPresenter: AlertPresenter

    weak var delegate: IPOverrideViewControllerDelegate?

    private lazy var containerView: UIStackView = {
        let view = UIStackView()
        view.axis = .vertical
        view.spacing = 20
        return view
    }()

    private lazy var clearButton: AppButton = {
        let button = AppButton(style: .danger)
        button.addTarget(self, action: #selector(didTapClearButton), for: .touchUpInside)
        button.setTitle(NSLocalizedString(
            "IP_OVERRIDE_CLEAR_BUTTON",
            tableName: "IPOverride",
            value: "Clear all overrides",
            comment: ""
        ), for: .normal)
        return button
    }()

    private let statusView = IPOverrideStatusView()

    init(interactor: IPOverrideInteractor, alertPresenter: AlertPresenter) {
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
        view.directionalLayoutMargins = UIMetrics.contentHeadingLayoutMargins

        configureNavigation()
        addHeaderView()
        addImportButtons()
        addStatusLabel()

        view.addConstrainedSubviews([containerView, clearButton]) {
            containerView.pinEdgesToSuperviewMargins(.all().excluding(.bottom))
            clearButton.pinEdgesToSuperviewMargins(PinnableEdges([.leading(0), .trailing(0), .bottom(16)]))
        }

        interactor.statusPublisher.sink { [weak self] status in
            self?.statusView.setStatus(status)
            self?.clearButton.isEnabled = self?.interactor.defaultStatus == .active
        }.store(in: &cancellables)
    }

    private func configureNavigation() {
        title = NSLocalizedString(
            "IP_OVERRIDE_HEADER",
            tableName: "IPOverride",
            value: "Server IP override",
            comment: ""
        )
    }

    private func addHeaderView() {
        let body = NSLocalizedString(
            "IP_OVERRIDE_HEADER_BODY",
            tableName: "IPOverride",
            value: "Import files or text with the new IP addresses for the servers in the Select location view.",
            comment: ""
        )
        let link = NSLocalizedString(
            "IP_OVERRIDE_HEADER_LINK",
            tableName: "IPOverride",
            value: "About Server IP override...",
            comment: ""
        )

        let headerView = InfoHeaderView(config: InfoHeaderConfig(body: body, link: link))

        headerView.onAbout = { [weak self] in
            self?.delegate?.presentAbout()
        }

        containerView.addArrangedSubview(headerView)
    }

    private func addImportButtons() {
        let importTextButton = AppButton(style: .default)
        importTextButton.addTarget(self, action: #selector(didTapImportTextButton), for: .touchUpInside)
        importTextButton.setTitle(NSLocalizedString(
            "IP_OVERRIDE_IMPORT_TEXT_BUTTON",
            tableName: "IPOverride",
            value: "Import via text",
            comment: ""
        ), for: .normal)
        importTextButton.titleLabel?.textAlignment = .center

        let importFileButton = AppButton(style: .default)
        importFileButton.addTarget(self, action: #selector(didTapImportFileButton), for: .touchUpInside)
        importFileButton.setTitle(NSLocalizedString(
            "IP_OVERRIDE_IMPORT_FILE_BUTTON",
            tableName: "IPOverride",
            value: "Import file",
            comment: ""
        ), for: .normal)
        importFileButton.titleLabel?.textAlignment = .center
        let stackView = UIStackView(arrangedSubviews: [importTextButton, importFileButton])
        stackView.distribution = .fillEqually
        stackView.spacing = 12

        containerView.addArrangedSubview(stackView)
    }

    private func addStatusLabel() {
        containerView.addArrangedSubview(statusView)
    }

    @objc private func didTapClearButton() {
        let presentation = AlertPresentation(
            id: "ip-override-clear-alert",
            icon: .alert,
            title: NSLocalizedString(
                "IP_OVERRIDE_CLEAR_DIALOG_TITLE",
                tableName: "IPOverride",
                value: "Clear all overrides?",
                comment: ""
            ),
            message: NSLocalizedString(
                "IP_OVERRIDE_CLEAR_DIALOG_MESSAGE",
                tableName: "IPOverride",
                value: """
                Clearing the imported overrides changes the server IPs, in the Select location view, \
                back to default.
                """,
                comment: ""
            ),
            buttons: [
                AlertAction(
                    title: NSLocalizedString(
                        "IP_OVERRIDE_CLEAR_DIALOG_CLEAR_BUTTON",
                        tableName: "IPOverride",
                        value: "Clear",
                        comment: ""
                    ),
                    style: .destructive,
                    handler: { [weak self] in
                        self?.interactor.deleteAllOverrides()
                    }
                ),
                AlertAction(
                    title: NSLocalizedString(
                        "IP_OVERRIDE_CLEAR_DIALOG_CANCEL_BUTTON",
                        tableName: "IPOverride",
                        value: "Cancel",
                        comment: ""
                    ),
                    style: .default
                ),
            ]
        )

        alertPresenter.showAlert(presentation: presentation, animated: true)
    }

    @objc private func didTapImportTextButton() {
        delegate?.presentImportTextController()
    }

    @objc private func didTapImportFileButton() {
        let documentPicker = UIDocumentPickerViewController(forOpeningContentTypes: [.json, .text])
        documentPicker.delegate = self

        present(documentPicker, animated: true)
    }
}

extension IPOverrideViewController: UIDocumentPickerDelegate {
    func documentPicker(_ controller: UIDocumentPickerViewController, didPickDocumentsAt urls: [URL]) {
        if let url = urls.first {
            url.securelyScoped { [weak self] scopedUrl in
                scopedUrl.flatMap { self?.interactor.import(url: $0) }
            }
        }
    }
}
