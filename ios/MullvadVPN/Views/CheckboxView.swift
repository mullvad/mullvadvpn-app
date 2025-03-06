//
//  CheckboxView.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-06-05.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import UIKit

class CheckboxView: UIView {
    private let backgroundView: UIView = {
        let view = UIView()
        view.backgroundColor = .white
        view.layer.cornerRadius = 4
        return view
    }()

    private let checkmarkView: UIImageView = {
        let imageView = UIImageView(image: UIImage.tick)
        imageView.tintColor = .successColor
        imageView.contentMode = .scaleAspectFit
        imageView.alpha = 0
        return imageView
    }()

    var isChecked = false {
        didSet {
            checkmarkView.alpha = isChecked ? 1 : 0
        }
    }

    init() {
        super.init(frame: .zero)

        directionalLayoutMargins = .init(top: 4, leading: 4, bottom: 4, trailing: 4)

        addConstrainedSubviews([backgroundView, checkmarkView]) {
            backgroundView.pinEdgesToSuperview()
            checkmarkView.pinEdgesToSuperviewMargins()
        }
    }

    required init?(coder aDecoder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }
}
