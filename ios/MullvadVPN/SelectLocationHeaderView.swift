//
//  SelectLocationHeaderView.swift
//  MullvadVPN
//
//  Created by pronebird on 22/07/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import UIKit

class SelectLocationHeaderView: UIView {

    let textLabel = UILabel()

    override init(frame: CGRect) {
        super.init(frame: frame)

        layoutMargins = UIEdgeInsets(top: 24, left: 24, bottom: 24, right: 24)

        textLabel.translatesAutoresizingMaskIntoConstraints = false
        textLabel.font = UIFont.systemFont(ofSize: 17)
        textLabel.textColor = UIColor(white: 1, alpha: 0.6)
        textLabel.numberOfLines = 0
        textLabel.text = NSLocalizedString("While connected, your real location is masked with a private and secure location in the selected region", comment: "")

        addSubview(textLabel)

        NSLayoutConstraint.activate([
            textLabel.topAnchor.constraint(equalTo: layoutMarginsGuide.topAnchor),
            textLabel.leadingAnchor.constraint(equalTo: layoutMarginsGuide.leadingAnchor),
            textLabel.trailingAnchor.constraint(equalTo: layoutMarginsGuide.trailingAnchor),
            textLabel.bottomAnchor.constraint(equalTo: layoutMarginsGuide.bottomAnchor),
            textLabel.bottomAnchor.constraint(equalTo: layoutMarginsGuide.bottomAnchor)
        ])
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

}
