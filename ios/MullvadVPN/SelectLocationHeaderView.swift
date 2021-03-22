//
//  SelectLocationHeaderView.swift
//  MullvadVPN
//
//  Created by pronebird on 22/07/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import UIKit

class SelectLocationHeaderView: UITableViewHeaderFooterView {

    lazy var textContentLabel: UILabel = {
        let textLabel = UILabel()
        textLabel.translatesAutoresizingMaskIntoConstraints = false
        textLabel.font = UIFont.systemFont(ofSize: 17)
        textLabel.textColor = UIColor(white: 1, alpha: 0.6)
        textLabel.numberOfLines = 0
        textLabel.text = NSLocalizedString("While connected, your real location is masked with a private and secure location in the selected region", comment: "")
        return textLabel
    }()

    var topLayoutMarginAdjustmentForNavigationBarTitle: CGFloat = 0 {
        didSet {
            let value = UIMetrics.contentLayoutMargins.top - topLayoutMarginAdjustmentForNavigationBarTitle
            contentView.layoutMargins.top = max(value, 0)
        }
    }

    override init(reuseIdentifier: String?) {
        super.init(reuseIdentifier: reuseIdentifier)

        layoutMargins = .zero
        contentView.layoutMargins = UIMetrics.contentLayoutMargins
        contentView.addSubview(textContentLabel)

        backgroundView = UIView()
        backgroundView?.backgroundColor = .secondaryColor

        let trailingConstraint = textContentLabel.trailingAnchor.constraint(equalTo: contentView.layoutMarginsGuide.trailingAnchor)
        trailingConstraint.priority = UILayoutPriority(800)

        let bottomConstraint = textContentLabel.bottomAnchor.constraint(equalTo: contentView.layoutMarginsGuide.bottomAnchor)
        bottomConstraint.priority = UILayoutPriority(800)

        NSLayoutConstraint.activate([
            textContentLabel.topAnchor.constraint(equalTo: contentView.layoutMarginsGuide.topAnchor),
            textContentLabel.leadingAnchor.constraint(equalTo: contentView.layoutMarginsGuide.leadingAnchor),
            trailingConstraint,
            bottomConstraint
        ])
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

}
