//
//  HeaderBarView.swift
//  MullvadVPN
//
//  Created by pronebird on 19/06/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation
import UIKit

class HeaderBarView: UIView {
    @IBOutlet var contentView: UIView!
    @IBOutlet var settingsButton: UIButton!

    init(bundle: Bundle?) {
        super.init(frame: .zero)

        loadFromNib(bundle: bundle)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    private func loadFromNib(bundle: Bundle?) {
        let nib = UINib(nibName: "HeaderBarView", bundle: bundle)
        _ = nib.instantiate(withOwner: self, options: nil)

        let constraints = [contentView.leadingAnchor.constraint(equalTo: leadingAnchor),
                           contentView.trailingAnchor.constraint(equalTo: trailingAnchor),
                           contentView.topAnchor.constraint(equalTo: topAnchor),
                           contentView.bottomAnchor.constraint(equalTo: bottomAnchor)]

        contentView.translatesAutoresizingMaskIntoConstraints = false
        addSubview(contentView)

        NSLayoutConstraint.activate(constraints)
    }
}
