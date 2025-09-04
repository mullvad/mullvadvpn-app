import { colorTokens } from '../tokens';

export const colorPrimitives = {
  '--color-white': colorTokens.white,
  '--color-white-alpha80': colorTokens.whiteAlpha80,
  '--color-white-alpha60': colorTokens.whiteAlpha60,
  '--color-white-alpha40': colorTokens.whiteAlpha40,
  '--color-white-alpha20': colorTokens.whiteAlpha20,

  '--color-black': colorTokens.black,
  '--color-black-alpha50': colorTokens.blackAlpha50,

  '--color-red': colorTokens.red,
  '--color-new-red': colorTokens.newRed,
  '--color-red-alpha40': colorTokens.redAlpha40,
  '--color-red80': colorTokens.red80,
  '--color-red40': colorTokens.red40,

  '--color-green': colorTokens.green,
  '--color-green-alpha40': colorTokens.greenAlpha40,
  '--color-green80': colorTokens.green80,
  '--color-green40': colorTokens.green40,

  '--color-yellow': colorTokens.yellow,
  '--color-fur': colorTokens.fur,
  '--color-nose': colorTokens.nose,
  '--color-blue': colorTokens.blue,
  '--color-dark-blue': colorTokens.darkBlue,

  '--color-dark': colorTokens.dark,
  '--color-darker-blue50': colorTokens.darkerBlue50,
  '--color-darker-blue50-alpha80': colorTokens.darkerBlue50Alpha80,
  '--color-darker-blue10': colorTokens.darkerBlue10,
  '--color-darker-blue10-alpha80': colorTokens.darkerBlue10Alpha80,
  '--color-darker-blue10-alpha40': colorTokens.darkerBlue10Alpha40,

  '--color-blue10': colorTokens.blue10,
  '--color-blue20': colorTokens.blue20,
  '--color-blue40': colorTokens.blue40,
  '--color-blue50': colorTokens.blue50,
  '--color-blue60': colorTokens.blue60,
  '--color-blue80': colorTokens.blue80,

  '--color-white-on-dark-blue5': colorTokens.whiteOnDarkBlue5,
  '--color-white-on-dark-blue10': colorTokens.whiteOnDarkBlue10,
  '--color-white-on-dark-blue20': colorTokens.whiteOnDarkBlue20,
  '--color-white-on-dark-blue40': colorTokens.whiteOnDarkBlue40,
  '--color-white-on-dark-blue50': colorTokens.whiteOnDarkBlue50,
  '--color-white-on-dark-blue60': colorTokens.whiteOnDarkBlue60,
  '--color-white-on-dark-blue80': colorTokens.whiteOnDarkBlue80,

  '--color-white-on-blue5': colorTokens.whiteOnBlue5,
  '--color-white-on-blue10': colorTokens.whiteOnBlue10,
  '--color-white-on-blue20': colorTokens.whiteOnBlue20,
  '--color-white-on-blue40': colorTokens.whiteOnBlue40,
  '--color-white-on-blue50': colorTokens.whiteOnBlue50,
  '--color-white-on-blue60': colorTokens.whiteOnBlue60,
  '--color-white-on-blue80': colorTokens.whiteOnBlue80,

  '--color-chalk': colorTokens.chalk,
  '--color-chalk-alpha80': colorTokens.chalkAlpha80,
  '--color-chalk-alpha40': colorTokens.chalkAlpha40,
  '--color-chalk80': colorTokens.chalk80,

  '--transparent': 'transparent',
} as const;

export const colors: Record<keyof typeof colorTokens, `var(${keyof typeof colorPrimitives})`> = {
  white: 'var(--color-white)',
  whiteAlpha80: 'var(--color-white-alpha80)',
  whiteAlpha60: 'var(--color-white-alpha60)',
  whiteAlpha40: 'var(--color-white-alpha40)',
  whiteAlpha20: 'var(--color-white-alpha20)',

  black: 'var(--color-black)',
  blackAlpha50: 'var(--color-black-alpha50)',

  red: 'var(--color-red)',
  newRed: 'var(--color-new-red)',
  redAlpha40: 'var(--color-red-alpha40)',
  red80: 'var(--color-red80)',
  red40: 'var(--color-red40)',

  green: 'var(--color-green)',
  greenAlpha40: 'var(--color-green-alpha40)',
  green80: 'var(--color-green80)',
  green40: 'var(--color-green40)',

  yellow: 'var(--color-yellow)',
  fur: 'var(--color-fur)',
  nose: 'var(--color-nose)',
  blue: 'var(--color-blue)',
  darkBlue: 'var(--color-dark-blue)',

  dark: 'var(--color-dark)',
  darkerBlue50: 'var(--color-darker-blue50)',
  darkerBlue10: 'var(--color-darker-blue10)',
  darkerBlue10Alpha80: 'var(--color-darker-blue10-alpha80)',
  darkerBlue10Alpha40: 'var(--color-darker-blue10-alpha40)',
  darkerBlue50Alpha80: 'var(--color-darker-blue50-alpha80)',

  blue10: 'var(--color-blue10)',
  blue20: 'var(--color-blue20)',
  blue40: 'var(--color-blue40)',
  blue50: 'var(--color-blue50)',
  blue60: 'var(--color-blue60)',
  blue80: 'var(--color-blue80)',

  whiteOnDarkBlue5: 'var(--color-white-on-dark-blue5)',
  whiteOnDarkBlue10: 'var(--color-white-on-dark-blue10)',
  whiteOnDarkBlue20: 'var(--color-white-on-dark-blue20)',
  whiteOnDarkBlue40: 'var(--color-white-on-dark-blue40)',
  whiteOnDarkBlue50: 'var(--color-white-on-dark-blue50)',
  whiteOnDarkBlue60: 'var(--color-white-on-dark-blue60)',
  whiteOnDarkBlue80: 'var(--color-white-on-dark-blue80)',

  whiteOnBlue5: 'var(--color-white-on-blue5)',
  whiteOnBlue10: 'var(--color-white-on-blue10)',
  whiteOnBlue20: 'var(--color-white-on-blue20)',
  whiteOnBlue40: 'var(--color-white-on-blue40)',
  whiteOnBlue50: 'var(--color-white-on-blue50)',
  whiteOnBlue60: 'var(--color-white-on-blue60)',
  whiteOnBlue80: 'var(--color-white-on-blue80)',

  chalk: 'var(--color-chalk)',
  chalkAlpha40: 'var(--color-chalk-alpha40)',
  chalkAlpha80: 'var(--color-chalk-alpha80)',
  chalk80: 'var(--color-chalk80)',

  transparent: 'var(--transparent)',
} as const;

export type Colors = keyof typeof colors;

export type ColorVariables = (typeof colors)[keyof typeof colors];
