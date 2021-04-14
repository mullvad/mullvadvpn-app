# Graphic assets

This folder contains graphic assets that are used to generate assets for either the app or platforms
where the app is distributed.

## Android

The `Android-feature-graphics.psd` file should be used to generate a PNG image to be used as the
feature graphics in the app's Google Play Store listing. The PNG image should be placed in the
`android/src/main/play/listings/en-US/graphics/feature-graphics/` directory.

The `icon-square.svg` is used to generate Android's square icon used in the app's Google Play Store
listing. The resulting 512x512 PNG image should be placed in the
`android/src/main/play/listings/en-US/graphics/icon/` directory. The file can be generate with the
following command:

```
rsvg-convert ./icon-square.svg -w 512 -h 512 -o ../android/src/main/play/listings/en-US/graphics/icon/icon.png
```

The icon `adaptive-icon-source.svg` is used for Android adaptive icon. The icon converted to
Android Vector Drawable format and used as foreground layer for adaptive icon. For background layer is used
solid color layer. Full documentation about adaptive icon available on link below:
https://developer.android.com/guide/practices/ui_guidelines/icon_design_adaptive
