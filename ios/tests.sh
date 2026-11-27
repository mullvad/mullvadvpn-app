xcodebuild test \
    -scheme "MullvadVPNUITests" \
    -destination "platform=iOS,id=00008110-001439E822A2401E" \
    -only-testing:MullvadVPNUITests/PaymentTests/testMakeInAppPurchaseOnAccountScreen \
    -only-testing:MullvadVPNUITests/PaymentTests/testMakeInAppPurchaseOnWelcomeScreen \
    -only-testing:MullvadVPNUITests/PaymentTests/testMakeInAppPurchaseOnOutOfTimeScreen \
    -only-testing:MullvadVPNUITests/PaymentTests/testInAppPurchaseWithRestoreOnFailedReceiptUpload
xcodebuild test-without-building \
    -scheme "MullvadVPNUITests" \
    -destination "platform=iOS,id=00008110-001439E822A2401E" \
    -only-testing:MullvadVPNUITests/RelayTests/testAdBlockingViaDNS