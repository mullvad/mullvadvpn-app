//
//  MethodTestingStatusContentCell.swift
//  MullvadVPN
//
//  Created by pronebird on 27/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

/// Content view presenting the access method testing progress.
class MethodTestingStatusCellContentView: UIView, UIContentView {
    var configuration: UIContentConfiguration {
        get {
            actualConfiguration
        }
        set {
            guard let newConfiguration = newValue as? MethodTestingStatusCellContentConfiguration,
                  actualConfiguration != newConfiguration else { return }

            let previousConfiguration = actualConfiguration
            actualConfiguration = newConfiguration

            configureSubviews(previousConfiguration: previousConfiguration)
        }
    }

    private var actualConfiguration: MethodTestingStatusCellContentConfiguration
    private let sheetContentView = AccessMethodActionSheetContentView()

    func supports(_ configuration: UIContentConfiguration) -> Bool {
        configuration is MethodTestingStatusCellContentConfiguration
    }

    init(configuration: MethodTestingStatusCellContentConfiguration) {
        actualConfiguration = configuration

        super.init(frame: CGRect(x: 0, y: 0, width: 100, height: 44))

        configureSubviews()
        addSubviews()
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    private func addSubviews() {
        addConstrainedSubviews([sheetContentView]) {
            sheetContentView.pinEdgesToSuperviewMargins()
        }
    }

    private func configureSubviews(previousConfiguration: MethodTestingStatusCellContentConfiguration? = nil) {
        configureLayoutMargins()
        configureSheetContentView()
    }

    private func configureLayoutMargins() {
        directionalLayoutMargins = actualConfiguration.directionalLayoutMargins
    }

    private func configureSheetContentView() {
        sheetContentView.configuration = actualConfiguration.sheetConfiguration
    }
}
