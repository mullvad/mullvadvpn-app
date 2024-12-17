import { FontFamilyTokens, FontSizeTokens, FontWeightTokens, LineHeightTokens } from '../tokens';

export const fontFamilies = {
  '--font-family-open-sans': FontFamilyTokens.openSans,
  '--font-family-source-sans-pro': FontFamilyTokens.sourceSansPro,
};

export enum FontFamilies {
  openSans = 'var(--font-family-open-sans)',
  sourceSansPro = 'var(--font-family-source-sans-pro)',
}

export const fontWeights = {
  '--font-weight-regular': FontWeightTokens.regular,
  '--font-weight-semi-bold': FontWeightTokens.semiBold,
  '--font-weight-bold': FontWeightTokens.bold,
};

export enum FontWeights {
  regular = 'var(--font-weight-regular)',
  semiBold = 'var(--font-weight-semi-bold)',
  bold = 'var(--font-weight-bold)',
}

export const fontSizes = {
  '--font-size-big': FontSizeTokens.big,
  '--font-size-large': FontSizeTokens.large,
  '--font-size-medium': FontSizeTokens.medium,
  '--font-size-small': FontSizeTokens.small,
  '--font-size-tiny': FontSizeTokens.tiny,
  '--font-size-mini': FontSizeTokens.mini,
};

export enum FontSizes {
  big = 'var(--font-size-big)',
  large = 'var(--font-size-large)',
  medium = 'var(--font-size-medium)',
  small = 'var(--font-size-small)',
  tiny = 'var(--font-size-tiny)',
  mini = 'var(--font-size-mini)',
}

export const lineHeights = {
  '--line-height-big': LineHeightTokens.big,
  '--line-height-large': LineHeightTokens.large,
  '--line-height-medium': LineHeightTokens.medium,
  '--line-height-small': LineHeightTokens.small,
  '--line-height-tiny': LineHeightTokens.tiny,
  '--line-height-mini': LineHeightTokens.mini,
};

export enum LineHeights {
  big = 'var(--line-height-big)',
  large = 'var(--line-height-large)',
  medium = 'var(--line-height-medium)',
  small = 'var(--line-height-small)',
  tiny = 'var(--line-height-tiny)',
  mini = 'var(--line-height-mini)',
}

export enum Fonts {
  title = FontFamilies.sourceSansPro,
  body = FontFamilies.openSans,
  label = FontFamilies.openSans,
  footnote = FontFamilies.openSans,
}

export type Typography =
  | 'titleBig'
  | 'titleLarge'
  | 'titleMedium'
  | 'bodySmall'
  | 'bodySmallSemibold'
  | 'labelTiny'
  | 'footnoteMini';

export interface TypographyProperties {
  fontFamily: React.CSSProperties['fontFamily'];
  fontSize: React.CSSProperties['fontSize'];
  fontWeight: React.CSSProperties['fontWeight'];
  lineHeight: React.CSSProperties['lineHeight'];
}

export const typography: Record<Typography, TypographyProperties> = {
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
