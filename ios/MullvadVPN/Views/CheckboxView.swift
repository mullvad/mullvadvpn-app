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
        view.layer.borderColor = UIColor.white.cgColor
        view.layer.borderWidth = 2
        view.layer.cornerRadius = 4
        return view
    }()

    private let checkmarkView: UIImageView = {
        let imageView = UIImageView(image: UIImage.tickSmall)
        imageView.tintColor = .successColor
        imageView.contentMode = .scaleAspectFit
        imageView.alpha = 0
        return imageView
    }()

    var isChecked = false {
        didSet {
            backgroundView.backgroundColor = isChecked ? .white : .clear
            checkmarkView.alpha = isChecked ? 1 : 0
        }
    }

    init() {
        super.init(frame: .zero)
        addConstrainedSubviews([backgroundView, checkmarkView]) {
            backgroundView.pinEdgesToSuperview()
            checkmarkView.pinEdgesToSuperview()
        }
    }

    required init?(coder aDecoder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }
}
