import React from 'react';

import { colors } from '../../config.json';

export const openSans: React.CSSProperties = {
  fontFamily: 'Open Sans',
};

export const sourceSansPro: React.CSSProperties = {
  fontFamily: '"Source Sans Pro", "Noto Sans Myanmar", "Noto Sans Thai", sans-serif',
};

export const tinyText = {
  ...openSans,
  fontSize: '12px',
  fontWeight: 600,
  lineHeight: '18px',
};

export const smallText = {
  ...openSans,
  fontSize: '14px',
  fontWeight: 600,
  lineHeight: '20px',
  color: colors.white80,
};

export const smallNormalText = {
  ...smallText,
  fontWeight: 'normal',
};

export const normalText = {
  ...openSans,
  fontSize: '15px',
  lineHeight: '18px',
};

export const largeText = {
  ...sourceSansPro,
  fontWeight: 600,
  fontSize: '18px',
  lineHeight: '24px',
};

export const buttonText = {
  ...largeText,
  color: colors.white,
};

export const bigText = {
  ...sourceSansPro,
  fontSize: '24px',
  fontWeight: 700,
  lineHeight: '28px',
};

export const hugeText = {
  ...sourceSansPro,
  fontSize: '32px',
  fontWeight: 700,
  lineHeight: '34px',
  color: colors.white,
};

export const spacings = {
  spacing1: '4px',
  spacing2: '6px',
  spacing3: '8px',
  spacing4: '12px',
  spacing5: '16px',
  spacing6: '24px',
  spacing7: '32px',
  spacing8: '40px',
  spacing9: '48px',
  spacing10: '56px',
  spacing11: '64px',
  spacing12: '72px',
  spacing13: '80px',
};

export const measurements = {
  rowMinHeight: spacings.spacing9,
  horizontalViewMargin: spacings.spacing6,
  verticalViewMargin: spacings.spacing5,
  rowVerticalMargin: spacings.spacing6,
  buttonVerticalMargin: spacings.spacing5,
};
