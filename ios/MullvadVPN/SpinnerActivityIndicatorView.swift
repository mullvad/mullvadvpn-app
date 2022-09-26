//
//  SpinnerActivityIndicatorView.swift
//  MullvadVPN
//
//  Created by pronebird on 15/05/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import UIKit

class SpinnerActivityIndicatorView: UIView {
    private static let rotationAnimationKey = "rotation"
    private static let animationDuration = 0.6

    enum Style {
        case small, medium, large, custom

        var intrinsicSize: CGSize {
            switch self {
            case .small:
                return CGSize(width: 16, height: 16)
            case .medium:
                return CGSize(width: 20, height: 20)
            case .large:
                return CGSize(width: 60, height: 60)
            case .custom:
                return CGSize(width: UIView.noIntrinsicMetric, height: UIView.noIntrinsicMetric)
            }
        }
    }

    private let imageView = UIImageView(image: UIImage(named: "IconSpinner"))

    private(set) var isAnimating = false
    private(set) var style = Style.large

    private var sceneActivationObserver: Any?

    override var intrinsicContentSize: CGSize {
        return style.intrinsicSize
    }

    init(style: Style) {
        self.style = style

        let size = style == .custom ? .zero : style.intrinsicSize

        super.init(frame: CGRect(origin: .zero, size: size))

        addSubview(imageView)
        isHidden = true
        backgroundColor = UIColor.clear
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    deinit {
        unregisterSceneActivationObserver()
    }

    override func didMoveToWindow() {
        super.didMoveToWindow()

        if window == nil {
            unregisterSceneActivationObserver()
        } else {
            registerSceneActivationObserver()
            restartAnimationIfNeeded()
        }
    }

    override func layoutSubviews() {
        super.layoutSubviews()

        let size = style == .custom ? frame.size : style.intrinsicSize

        imageView.frame = CGRect(origin: .zero, size: size)
    }

    func startAnimating() {
        guard !isAnimating else { return }
        isAnimating = true

        isHidden = false
        addAnimation()
    }

    func stopAnimating() {
        guard isAnimating else { return }
        isAnimating = false

        isHidden = true
        removeAnimation()
    }

    private func addAnimation() {
        layer.add(createAnimation(), forKey: Self.rotationAnimationKey)
    }

    private func removeAnimation() {
        layer.removeAnimation(forKey: Self.rotationAnimationKey)
    }

    private func registerSceneActivationObserver() {
        unregisterSceneActivationObserver()

        sceneActivationObserver = NotificationCenter.default.addObserver(
            forName: UIScene.willEnterForegroundNotification,
            object: window?.windowScene,
            queue: .main, using: { [weak self] _ in
                self?.restartAnimationIfNeeded()
            }
        )
    }

    private func unregisterSceneActivationObserver() {
        if let sceneActivationObserver = sceneActivationObserver {
            NotificationCenter.default.removeObserver(sceneActivationObserver)
            self.sceneActivationObserver = nil
        }
    }

    private func restartAnimationIfNeeded() {
        let animation = layer.animation(forKey: Self.rotationAnimationKey)

        if isAnimating, animation == nil {
            removeAnimation()
            addAnimation()
        }
    }

    private func createAnimation() -> CABasicAnimation {
        let animation = CABasicAnimation(keyPath: "transform.rotation")
        animation.toValue = NSNumber(value: Double.pi * 2)
        animation.duration = Self.animationDuration
        animation.repeatCount = Float.infinity
        animation.timingFunction = CAMediaTimingFunction(name: .linear)
        animation.timeOffset = layer.convertTime(CACurrentMediaTime(), from: nil)

        return animation
    }
}
