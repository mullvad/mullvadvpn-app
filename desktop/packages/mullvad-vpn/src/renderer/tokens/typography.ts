export enum FontFamilies {
  openSans = 'Open Sans',
  openSansPro = '"Source Sans Pro", "Noto Sans Myanmar", "Noto Sans Thai", sans-serif',
}

export enum Fonts {
  title = FontFamilies.openSansPro,
  body = FontFamilies.openSans,
  label = FontFamilies.openSans,
  footnote = FontFamilies.openSans,
}

export enum FontWeights {
  regular = 400,
  semiBold = 600,
  bold = 700,
}

export enum FontSizes {
  big = '32px',
  large = '24px',
  medium = '18px',
  small = '14px',
  tiny = '12px',
  mini = '10px',
}

export enum LineHeights {
  big = '34px',
  large = '28px',
  medium = '24px',
  small = '20px',
  tiny = '18px',
  mini = '15px',
}

interface Typography {
  fontFamily: React.CSSProperties['fontFamily'];
  fontSize: React.CSSProperties['fontSize'];
  fontWeight: React.CSSProperties['fontWeight'];
  lineHeight: React.CSSProperties['lineHeight'];
}

export const typography: Record<
  | 'titleBig'
  | 'titleLarge'
  | 'titleMedium'
  | 'bodySmall'
  | 'bodySmallSemibold'
  | 'labelTiny'
  | 'footnoteMini',
  Typography
> = {
  titleBig: {
    fontFamily: Fonts.title,
    fontWeight: FontWeights.bold,
    fontSize: FontSizes.big,
    lineHeight: LineHeights.big,
  },
  titleLarge: {
    fontFamily: Fonts.title,
    fontWeight: FontWeights.bold,
    fontSize: FontSizes.large,
    lineHeight: LineHeights.large,
  },
  titleMedium: {
    fontFamily: Fonts.title,
    fontWeight: FontWeights.semiBold,
    fontSize: FontSizes.medium,
    lineHeight: LineHeights.medium,
  },
  bodySmall: {
    fontFamily: Fonts.body,
    fontWeight: FontWeights.regular,
    fontSize: FontSizes.small,
    lineHeight: LineHeights.small,
  },
  bodySmallSemibold: {
    fontFamily: Fonts.body,
    fontWeight: FontWeights.semiBold,
    fontSize: FontSizes.small,
    lineHeight: LineHeights.small,
  },
  labelTiny: {
    fontFamily: Fonts.label,
    fontWeight: FontWeights.semiBold,
    fontSize: FontSizes.tiny,
    lineHeight: LineHeights.tiny,
  },
  footnoteMini: {
    fontFamily: Fonts.footnote,
    fontWeight: FontWeights.semiBold,
    fontSize: FontSizes.mini,
    lineHeight: LineHeights.mini,
  },
} as const;
