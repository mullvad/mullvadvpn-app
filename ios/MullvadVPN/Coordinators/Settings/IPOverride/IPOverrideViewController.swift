//
//  IPOverrideViewController.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-01-15.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
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
        view.spacing = 16
        return view
    }()

    private lazy var clearButton: AppButton = {
        let button = AppButton(style: .danger)
        button.addTarget(self, action: #selector(didTapClearButton), for: .touchUpInside)
        button.setTitle(NSLocalizedString("Clear all overrides", comment: ""), for: .normal)
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
        configureNavigation()
        addHeaderView()
        addImportButtons()
        addStatusLabel()
        addScrollView()

        interactor.statusPublisher.sink { [weak self] status in
            self?.statusView.setStatus(status)
            self?.clearButton.isEnabled = self?.interactor.defaultStatus == .active
        }.store(in: &cancellables)
    }

    private func configureNavigation() {
        title = NSLocalizedString("Server IP override", comment: "")
    }

    private func addScrollView() {
        let scrollView = UIScrollView()
        let contentView = UIView()
        contentView.directionalLayoutMargins = UIMetrics.contentHeadingLayoutMargins

        view.addConstrainedSubviews([scrollView]) {
            scrollView.pinEdgesToSuperview()
        }

        scrollView.addConstrainedSubviews([contentView]) {
            contentView.pinEdgesToSuperview()
            contentView.widthAnchor.constraint(equalTo: scrollView.widthAnchor)
            contentView.heightAnchor.constraint(greaterThanOrEqualTo: view.layoutMarginsGuide.heightAnchor)
        }

        let spacer = UIView()

        let contentStackView = UIStackView(arrangedSubviews: [containerView, spacer, clearButton])
        contentStackView.axis = .vertical
        contentStackView.distribution = .fill
        contentStackView.spacing = 8

        contentView.addConstrainedSubviews([contentStackView]) {
            contentStackView.pinEdgesToSuperviewMargins()
        }

        // Hugging & resistance priorities
        spacer.setContentHuggingPriority(.defaultLow, for: .vertical)
        spacer.setContentCompressionResistancePriority(.defaultLow, for: .vertical)

        containerView.setContentHuggingPriority(.required, for: .vertical)
        containerView.setContentCompressionResistancePriority(.required, for: .vertical)

        clearButton.setContentHuggingPriority(.required, for: .vertical)
        clearButton.setContentCompressionResistancePriority(.required, for: .vertical)
    }

    private func addHeaderView() {
        let body = NSLocalizedString(
            "Import files or text with new IP addresses for the servers in the Select location view.",
            comment: ""
        )
        let link = NSLocalizedString("About Server IP override...", comment: "")

        let headerView = InfoHeaderView(config: InfoHeaderConfig(body: body, link: link))

        headerView.onAbout = { [weak self] in
            self?.delegate?.presentAbout()
        }

        containerView.addArrangedSubview(headerView)
    }

    private func addImportButtons() {
        let importTextButton = AppButton(style: .default)
        importTextButton.addTarget(self, action: #selector(didTapImportTextButton), for: .touchUpInside)
        importTextButton.setTitle(NSLocalizedString("Import via text", comment: ""), for: .normal)
        importTextButton.titleLabel?.textAlignment = .center

        let importFileButton = AppButton(style: .default)
        importFileButton.addTarget(self, action: #selector(didTapImportFileButton), for: .touchUpInside)
        importFileButton.setTitle(NSLocalizedString("Import file", comment: ""), for: .normal)
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
            title: NSLocalizedString("Clear all overrides?", comment: ""),
            message: NSLocalizedString(
                "Clearing the imported overrides changes the server IPs, in the Select location view, back to default.",
                comment: ""
            ),
            buttons: [
                AlertAction(
                    title: NSLocalizedString("Clear", comment: ""),
                    style: .destructive,
                    handler: { [weak self] in
                        self?.interactor.deleteAllOverrides()
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
