# Graphic assets

This folder contains graphic assets that are used to generate assets for either the app or platforms
where the app is distributed.

## Android

The `Android-feature-graphics.psd` file should be used to generate a PNG image to be used as the
feature graphics in the app's Google Play Store listing. The PNG image should be placed in the
`android/app/src/main/play/listings/en-US/graphics/feature-graphics/` directory.

## Icons (The mole logo in different versions)

### `icon.svg`

The main and official mole logo. Used to generate icons on a bunch of platforms.

### `icon-square.svg`

This is the regular mole but instead of being placed in a blue circle the entire background is just blue.
The mole is placed slighty to the right compared to `icon.svg` to appear more centered.

#### Android

The `icon-square.svg` is used to generate Android's square icon used in the app's Google Play Store
listing. The resulting 512x512 PNG image should be placed in the
`android/app/src/main/play/listings/en-US/graphics/icon/` directory. The file can be generate with the
following command:

```
rsvg-convert ./icon-square.svg -w 512 -h 512 -o ../android/app/src/main/play/listings/en-US/graphics/icon/icon.png
```

#### iOS

`icon-square.svg` is used to generate the app icon for iOS. This is done by `ios/convert-assets.rb`.

### `adaptive-icon-source.svg`

The icon `adaptive-icon-source.svg` is used for Android adaptive icon. The icon converted to
Android Vector Drawable format and used as foreground layer for adaptive icon. For background layer is used
solid color layer. Full documentation about adaptive icon available on link below:
https://developer.android.com/guide/practices/ui_guidelines/icon_design_adaptive

### `icon-shaved.svg`

This is a simplified version of the logo with the whiskers and fur removed. This version is used to generate
small icon assets for Android where the little details in the logo would not be visible anyway, and would
just make the small assets look less clean.

This file is used by `android/scripts/generate-pngs.sh`
