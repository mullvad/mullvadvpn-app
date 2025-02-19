//
//  AddCreditSucceededViewController.swift
//  MullvadVPN
//
//  Created by Sajad Vishkai on 2022-09-23.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import UIKit

protocol AddCreditSucceededViewControllerDelegate: AnyObject {
    func header(in controller: AddCreditSucceededViewController) -> String

    func titleForAction(in controller: AddCreditSucceededViewController) -> String

    func addCreditSucceededViewControllerDidFinish(in controller: AddCreditSucceededViewController)
}

class AddCreditSucceededViewController: UIViewController, RootContainment {
    private let statusImageView: StatusImageView = {
        let statusImageView = StatusImageView(style: .success)
        statusImageView.translatesAutoresizingMaskIntoConstraints = false
        return statusImageView
    }()

    private let titleLabel: UILabel = {
        let label = UILabel()
        label.font = UIFont.boldSystemFont(ofSize: 20)
        label.textColor = .white
        label.numberOfLines = 0
        label.translatesAutoresizingMaskIntoConstraints = false
        return label
    }()

    private let messageLabel: UILabel = {
        let label = UILabel()
        label.font = UIFont.systemFont(ofSize: 17)
        label.textColor = UIColor.white.withAlphaComponent(0.6)
        label.numberOfLines = 0
        label.translatesAutoresizingMaskIntoConstraints = false
        return label
    }()

    private let dismissButton: AppButton = {
        let button = AppButton(style: .default)
        button.translatesAutoresizingMaskIntoConstraints = false
        return button
    }()

    override var preferredStatusBarStyle: UIStatusBarStyle {
        .lightContent
    }

    var preferredHeaderBarPresentation: HeaderBarPresentation {
        HeaderBarPresentation(style: .default, showsDivider: true)
    }

    var prefersHeaderBarHidden: Bool {
        false
    }

    var prefersDeviceInfoBarHidden: Bool {
        true
    }

    weak var delegate: AddCreditSucceededViewControllerDelegate? {
        didSet {
            dismissButton.setTitle(delegate?.titleForAction(in: self), for: .normal)
            titleLabel.text = delegate?.header(in: self)
        }
    }

    init(timeAddedComponents: DateComponents) {
        super.init(nibName: nil, bundle: nil)

        view.backgroundColor = .secondaryColor
        view.directionalLayoutMargins = UIMetrics.contentLayoutMargins

        messageLabel.text = String(
            format: NSLocalizedString(
                "ADDED_TIME_SUCCESS_MESSAGE",
                tableName: "AddedTime",
                value: "%@ were added to your account.",
                comment: ""
            ),
            timeAddedComponents.formattedAddedDay
        )
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func viewDidLoad() {
        super.viewDidLoad()
        configureUI()
        addDismissButtonHandler()
    }

    private func configureUI() {
        let contentHolderView = UIView(frame: .zero)

        view.addConstrainedSubviews([contentHolderView]) {
            contentHolderView.pinEdgesToSuperview(.all(UIMetrics.SettingsRedeemVoucher.successfulRedeemMargins))
        }

        contentHolderView.addConstrainedSubviews([statusImageView, titleLabel, messageLabel, dismissButton]) {
            statusImageView.pinEdgesToSuperviewMargins(PinnableEdges([.top(0)]))
            statusImageView.centerXAnchor.constraint(equalTo: view.centerXAnchor)

            titleLabel.pinEdgesToSuperviewMargins(PinnableEdges([.leading(0), .trailing(0)]))
            titleLabel.topAnchor.constraint(
                equalTo: statusImageView.bottomAnchor,
                constant: UIMetrics.TableView.sectionSpacing
            )

            messageLabel.topAnchor.constraint(
                equalTo: titleLabel.layoutMarginsGuide.bottomAnchor,
                constant: UIMetrics.interButtonSpacing
            )
            messageLabel.pinEdgesToSuperviewMargins(PinnableEdges([.leading(0), .trailing(0)]))

            dismissButton.pinEdgesToSuperviewMargins(.all().excluding(.top))
        }
    }

    private func addDismissButtonHandler() {
        dismissButton.addTarget(
            self,
            action: #selector(handleDismissTap),
            for: .touchUpInside
        )
    }

    @objc private func handleDismissTap() {
        delegate?.addCreditSucceededViewControllerDidFinish(in: self)
    }
}

private extension DateComponents {
    var formattedAddedDay: String {
        let formatter = DateComponentsFormatter()
        formatter.allowedUnits = [.day]
        formatter.unitsStyle = .full
        return formatter.string(from: self) ?? ""
    }
}
