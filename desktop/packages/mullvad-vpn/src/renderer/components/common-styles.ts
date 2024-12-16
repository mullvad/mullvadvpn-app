import React from 'react';

import { Colors, Spacings } from './common/variables';

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
  color: Colors.white80,
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
  color: Colors.white,
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
  color: Colors.white,
};

export const measurements = {
  rowMinHeight: Spacings.spacing9,
  horizontalViewMargin: Spacings.spacing5,
  verticalViewMargin: Spacings.spacing6,
  rowVerticalMargin: Spacings.spacing6,
  buttonVerticalMargin: Spacings.spacing5,
};
