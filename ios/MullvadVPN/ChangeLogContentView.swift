//
//  ChangeLogContentView.swift
//  MullvadVPN
//
//  Created by Sajad Vishkai on 2022-12-14.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import UIKit

private let tableViewCellIdentifier = "ChangeLogTableCell.Identifier"

final class ChangeLogContentView: UIView, UITableViewDataSource {
    private var versionChanges: [String] = []

    private lazy var titleLabel: UILabel = {
        let label = UILabel()
        label.text = NSLocalizedString(
            "TITLE",
            tableName: "ChangeLog",
            value: "App version",
            comment: ""
        )
        label.textAlignment = .center
        label.font = UIFont.boldSystemFont(ofSize: 32)
        label.textColor = .white
        label.accessibilityIdentifier = "ChangeLogContentView.titleLabel"
        label.translatesAutoresizingMaskIntoConstraints = false
        return label
    }()

    private lazy var subtitleLabel: UILabel = {
        let label = UILabel()
        label.text = NSLocalizedString(
            "SUBTITLE",
            tableName: "ChangeLog",
            value: "Changes in this version:",
            comment: ""
        )
        label.font = UIFont.boldSystemFont(ofSize: 17)
        label.textColor = .white
        label.translatesAutoresizingMaskIntoConstraints = false
        label.accessibilityIdentifier = "ChangeLogContentView.subtitleLabel"
        return label
    }()

    private lazy var tableView: UITableView = {
        let tableView = UITableView()
        tableView.dataSource = self
        tableView.register(
            ChangeLogTableCellView.self,
            forCellReuseIdentifier: tableViewCellIdentifier
        )
        tableView.separatorStyle = .none
        tableView.backgroundColor = .clear
        tableView.rowHeight = UITableView.automaticDimension
        tableView.translatesAutoresizingMaskIntoConstraints = false
        return tableView
    }()

    private lazy var continueButton: AppButton = {
        let button = AppButton(style: .default)
        button.setTitle(
            NSLocalizedString(
                "CONTINUE_BUTTON",
                tableName: "ChangeLog",
                value: "Got it!",
                comment: ""
            ),
            for: .normal
        )
        button.accessibilityIdentifier = "ChangeLogContentView.continueButton"
        button.translatesAutoresizingMaskIntoConstraints = false
        return button
    }()

    var continueButtonAction: (() -> Void)?

    override init(frame: CGRect) {
        super.init(frame: frame)

        addViews()
        constraintViews()
        subscribeClicks()
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    public func setChangeDescriptions(_ descriptions: [String], animated: Bool = true) {
        versionChanges = descriptions
        tableView.reloadSections(IndexSet(integer: 0), with: animated ? .automatic : .none)
    }

    // MARK: - Private

    private func addViews() {
        [
            titleLabel,
            subtitleLabel,
            tableView,
            continueButton,
        ].forEach(addSubview)
    }

    private func constraintViews() {
        NSLayoutConstraint.activate([
            titleLabel.topAnchor.constraint(
                equalTo: topAnchor,
                constant: UIMetrics.contentLayoutMargins.top
            ),
            titleLabel.leadingAnchor.constraint(
                equalTo: leadingAnchor,
                constant: UIMetrics.contentLayoutMargins.left
            ),
            titleLabel.trailingAnchor.constraint(
                equalTo: trailingAnchor,
                constant: -UIMetrics.contentLayoutMargins.right
            ),

            subtitleLabel.topAnchor.constraint(
                equalTo: titleLabel.bottomAnchor,
                constant: UIMetrics.contentLayoutMargins.top
            ),
            subtitleLabel.leadingAnchor.constraint(
                equalTo: leadingAnchor,
                constant: UIMetrics.contentLayoutMargins.left
            ),
            subtitleLabel.trailingAnchor.constraint(
                equalTo: trailingAnchor,
                constant: -UIMetrics.contentLayoutMargins.right
            ),

            tableView.topAnchor.constraint(
                equalTo: subtitleLabel.bottomAnchor,
                constant: 8
            ),
            tableView.leadingAnchor.constraint(
                equalTo: leadingAnchor,
                constant: UIMetrics.contentLayoutMargins.left
            ),
            tableView.trailingAnchor.constraint(
                equalTo: trailingAnchor,
                constant: -UIMetrics.contentLayoutMargins.right
            ),

            continueButton.topAnchor.constraint(
                equalTo: tableView.bottomAnchor,
                constant: UIMetrics.contentLayoutMargins.top
            ),
            continueButton.leadingAnchor.constraint(
                equalTo: leadingAnchor,
                constant: UIMetrics.contentLayoutMargins.left
            ),
            continueButton.trailingAnchor.constraint(
                equalTo: trailingAnchor,
                constant: -UIMetrics.contentLayoutMargins.right
            ),
            continueButton.bottomAnchor.constraint(
                equalTo: bottomAnchor,
                constant: -UIMetrics.contentLayoutMargins.bottom
            ),
        ])
    }

    private func subscribeClicks() {
        continueButton.addTarget(
            self,
            action: #selector(continueButtonDidClicked),
            for: .touchUpInside
        )
    }

    @objc private func continueButtonDidClicked(_ button: AppButton) {
        continueButtonAction?()
    }

    // MARK: - TableView

    func tableView(_ tableView: UITableView, numberOfRowsInSection section: Int) -> Int {
        versionChanges.count
    }

    func tableView(_ tableView: UITableView, cellForRowAt indexPath: IndexPath) -> UITableViewCell {
        guard let cell = tableView.dequeueReusableCell(
            withIdentifier: tableViewCellIdentifier,
            for: indexPath
        ) as? ChangeLogTableCellView else { return UITableViewCell() }
        cell.setChange(versionChanges[indexPath.row])
        return cell
    }
}
