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

export const measurements = {
  rowMinHeight: '44px',
  viewMargin: '22px',
  rowVerticalMargin: '20px',
  buttonVerticalMargin: '18px',
};
