export const fontFamilies = {
  openSans: 'Open Sans',
  openSansPro: '"Source Sans Pro", "Noto Sans Myanmar", "Noto Sans Thai", sans-serif',
} as const;

export const fonts = {
  title: fontFamilies.openSansPro,
  body: fontFamilies.openSans,
  label: fontFamilies.openSans,
  footnote: fontFamilies.openSans,
};

export const fontWeights = {
  '400': 400,
  '600': 600,
  '700': 700,
} as const;

export const fontSizes = {
  big: '32px',
  large: '24px',
  medium: '18px',
  small: '14px',
  tiny: '12px',
  mini: '10px',
};

export const lineHeights = {
  big: '34px',
  large: '28px',
  medium: '24px',
  small: '20px',
  tiny: '18px',
  mini: '15px',
};

interface Typography {
  fontFamily: React.CSSProperties['fontFamily'];
  fontSize: React.CSSProperties['fontSize'];
  fontWeight: React.CSSProperties['fontWeight'];
  lineHeight: React.CSSProperties['lineHeight'];
}

export const typography: Record<
  | 'title-big'
  | 'title-large'
  | 'title-medium'
  | 'body-small'
  | 'body-small-semibold'
  | 'label-tiny'
  | 'footnote-mini',
  Typography
> = {
  'title-big': {
    fontFamily: fonts.title,
    fontWeight: fontWeights[700],
    fontSize: fontSizes.big,
    lineHeight: lineHeights.big,
  },
  'title-large': {
    fontFamily: fonts.title,
    fontWeight: fontWeights[700],
    fontSize: fontSizes.large,
    lineHeight: lineHeights.large,
  },
  'title-medium': {
    fontFamily: fonts.title,
    fontWeight: fontWeights[600],
    fontSize: fontSizes.medium,
    lineHeight: lineHeights.medium,
  },
  'body-small': {
    fontFamily: fonts.body,
    fontWeight: fontWeights[400],
    fontSize: fontSizes.small,
    lineHeight: lineHeights.small,
  },
  'body-small-semibold': {
    fontFamily: fonts.body,
    fontWeight: fontWeights[600],
    fontSize: fontSizes.small,
    lineHeight: lineHeights.small,
  },
  'label-tiny': {
    fontFamily: fonts.label,
    fontWeight: fontWeights[600],
    fontSize: fontSizes.tiny,
    lineHeight: lineHeights.tiny,
  },
  'footnote-mini': {
    fontFamily: fonts.footnote,
    fontWeight: fontWeights[600],
    fontSize: fontSizes.mini,
    lineHeight: lineHeights.mini,
  },
} as const;
