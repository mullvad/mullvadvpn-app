//
//  AccountContentView.swift
//  MullvadVPN
//
//  Created by pronebird on 08/07/2021.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import UIKit

class AccountContentView: UIView {
    let debugOptionsButton: AppButton = {
        let button = AppButton(style: .default)
        button.setAccessibilityIdentifier(.debugOptionsButton)
        button.setTitle(NSLocalizedString("Debug options", comment: ""), for: .normal)
        return button
    }()

    let purchaseButton: InAppPurchaseButton = {
        let button = InAppPurchaseButton()
        button.setAccessibilityIdentifier(.purchaseButton)
        button.setTitle(NSLocalizedString("Add time", comment: ""), for: .normal)
        return button
    }()

    let logoutButton: AppButton = {
        let button = AppButton(style: .danger)
        button.setAccessibilityIdentifier(.logoutButton)
        button.setTitle(NSLocalizedString("Log out", comment: ""), for: .normal)
        return button
    }()

    let deleteButton: AppButton = {
        let button = AppButton(style: .danger)
        button.setAccessibilityIdentifier(.deleteButton)
        button.setTitle(NSLocalizedString("Delete account", comment: ""), for: .normal)
        return button
    }()

    let accountDeviceRow: AccountDeviceRow = {
        AccountDeviceRow()
    }()

    let accountTokenRowView: AccountNumberRow = {
        AccountNumberRow()
    }()

    let accountExpiryRowView: AccountExpiryRow = {
        AccountExpiryRow()
    }()

    let restorePurchasesView: RestorePurchasesView = {
        RestorePurchasesView()
    }()

    lazy var contentStackView: UIStackView = {
        let stackView =
            UIStackView(arrangedSubviews: [
                accountDeviceRow,
                accountTokenRowView,
                accountExpiryRowView,
                restorePurchasesView,
            ])
        stackView.axis = .vertical
        stackView.spacing = UIMetrics.padding24
        stackView.setCustomSpacing(UIMetrics.padding8, after: accountExpiryRowView)
        return stackView
    }()

    lazy var buttonStackView: UIStackView = {
        var arrangedSubviews = [UIView]()
        #if DEBUG
            arrangedSubviews.append(debugOptionsButton)
        #endif
        arrangedSubviews.append(contentsOf: [
            purchaseButton,
            logoutButton,
            deleteButton,
        ])
        arrangedSubviews.forEach { $0.isExclusiveTouch = true }
        let stackView = UIStackView(arrangedSubviews: arrangedSubviews)
        stackView.axis = .vertical
        stackView.spacing = UIMetrics.padding16
        return stackView
    }()

    override init(frame: CGRect) {
        super.init(frame: frame)
        setAccessibilityIdentifier(.accountView)
        addScrollView()
    }

    private func addScrollView() {
        let scrollView = UIScrollView()
        let contentView = UIView()

        addConstrainedSubviews([scrollView]) {
            scrollView.pinEdgesToSuperviewMargins()
        }

        scrollView.addConstrainedSubviews([contentView]) {
            contentView.pinEdgesToSuperview()
            contentView.widthAnchor.constraint(equalTo: scrollView.widthAnchor)
            contentView.heightAnchor.constraint(greaterThanOrEqualTo: scrollView.frameLayoutGuide.heightAnchor)
        }

        let spacer = UIView()

        contentView.addConstrainedSubviews([contentStackView, spacer, buttonStackView]) {
            contentStackView.pinEdgesToSuperviewMargins(.all().excluding(.bottom))
            spacer.pinEdgesToSuperviewMargins(.all().excluding(.top).excluding(.bottom))
            buttonStackView.pinEdgesToSuperviewMargins(.all().excluding(.top))

            spacer.bottomAnchor.constraint(equalTo: buttonStackView.topAnchor)
            spacer.topAnchor.constraint(
                equalTo: contentStackView.bottomAnchor,
                constant: UIMetrics.TableView.sectionSpacing
            )
        }
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }
}
