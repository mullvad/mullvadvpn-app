//
//  AccessMethodActionSheetContainerView.swift
//  MullvadVPN
//
//  Created by pronebird on 15/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

/// The view implementing a vertical stack layout with the testing progress UI (content view) at the top and action buttons below.
class AccessMethodActionSheetContainerView: UIView {
    /// Sheet delegate.
    weak var delegate: AccessMethodActionSheetDelegate?

    /// Active configuration.
    var configuration = AccessMethodActionSheetConfiguration() {
        didSet {
            contentView.configuration = configuration.contentConfiguration
            updateView()
        }
    }

    private lazy var stackView: UIStackView = {
        let stackView = UIStackView(arrangedSubviews: [contentView, addButton, cancelButton])
        stackView.axis = .vertical
        stackView.spacing = UIMetrics.padding16
        return stackView
    }()

    private let contentView = AccessMethodActionSheetContentView()
    private let cancelButton: AppButton = {
        let button = AppButton(style: .tableInsetGroupedDefault)
        button.setTitle(
            NSLocalizedString("SHEET_CANCEL_BUTTON", tableName: "APIAccess", value: "Cancel", comment: ""),
            for: .normal
        )
        return button
    }()

    private let addButton: AppButton = {
        let button = AppButton(style: .tableInsetGroupedDanger)
        button.setTitle(
            NSLocalizedString("SHEET_ADD_ANYWAY_BUTTON", tableName: "APIAccess", value: "Add anyway", comment: ""),
            for: .normal
        )
        button.isHidden = true
        return button
    }()

    override init(frame: CGRect) {
        super.init(frame: frame)
        setupView()
        addActions()
        updateView()
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    private func setupView() {
        directionalLayoutMargins = UIMetrics.contentLayoutMargins

        addConstrainedSubviews([stackView]) {
            stackView.pinEdgesToSuperviewMargins()
        }
    }

    private func addActions() {
        let cancelAction = UIAction { [weak self] _ in
            self?.sendSheetDidCancel()
        }

        let addAction = UIAction { [weak self] _ in
            self?.sendSheetDidAdd()
        }

        cancelButton.addAction(cancelAction, for: .touchUpInside)
        addButton.addAction(addAction, for: .touchUpInside)
    }

    private func updateView() {
        let status = configuration.contentConfiguration.status

        switch configuration.context {
        case .addNew:
            addButton.isHidden = status != .unreachable
            cancelButton.isEnabled = status != .reachable

        case .proxyConfiguration:
            addButton.isHidden = true
            cancelButton.isEnabled = status == .testing
        }
    }

    private func sendSheetDidAdd() {
        delegate?.sheetDidAdd(self)
    }

    private func sendSheetDidCancel() {
        delegate?.sheetDidCancel(self)
    }
}
