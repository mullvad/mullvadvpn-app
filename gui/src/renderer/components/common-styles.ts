import { colors } from '../../config.json';

export const smallText = {
  fontFamily: 'Open Sans',
  fontSize: '13px',
  fontWeight: 600,
  lineHeight: '20px',
  color: colors.white80,
};

export const mediumText = {
  fontFamily: 'Open Sans',
  fontSize: '20px',
  lineHeight: '26px',
};

export const buttonText = {
  ...mediumText,
  fontFamily: 'DINPro',
  fontWeight: 900,
  color: colors.white,
};

export const bigText = {
  fontFamily: 'DINPro',
  fontSize: '32px',
  fontWeight: 900,
  lineHeight: '36px',
  color: colors.white,
};
