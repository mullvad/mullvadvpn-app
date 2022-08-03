//
//  SelectLocationHeaderView.swift
//  MullvadVPN
//
//  Created by pronebird on 22/07/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import UIKit

class SelectLocationHeaderView: UIView {
    lazy var textContentLabel: UILabel = {
        let textLabel = UILabel()
        textLabel.translatesAutoresizingMaskIntoConstraints = false
        textLabel.font = UIFont.systemFont(ofSize: 17)
        textLabel.textColor = UIColor(white: 1, alpha: 0.6)
        textLabel.numberOfLines = 0
        textLabel.text = NSLocalizedString(
            "SUBHEAD_LABEL",
            tableName: "SelectLocation",
            value: "While connected, your real location is masked with a private and secure location in the selected region",
            comment: ""
        )
        return textLabel
    }()

    var topLayoutMarginAdjustmentForNavigationBarTitle: CGFloat = 0 {
        didSet {
            let value = UIMetrics.sectionSpacing - topLayoutMarginAdjustmentForNavigationBarTitle
            layoutMargins.top = max(value, 0)
        }
    }

    init() {
        super.init(frame: CGRect(x: 0, y: 0, width: 100, height: 100))

        backgroundColor = .secondaryColor
        layoutMargins = UIMetrics.contentLayoutMargins
        insetsLayoutMarginsFromSafeArea = false

        addSubview(textContentLabel)

        NSLayoutConstraint.activate([
            textContentLabel.topAnchor.constraint(equalTo: layoutMarginsGuide.topAnchor),
            textContentLabel.leadingAnchor.constraint(equalTo: layoutMarginsGuide.leadingAnchor),
            textContentLabel.trailingAnchor.constraint(equalTo: layoutMarginsGuide.trailingAnchor),
            textContentLabel.bottomAnchor.constraint(equalTo: layoutMarginsGuide.bottomAnchor),
        ])
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }
}
