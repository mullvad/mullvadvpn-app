const namingConvention = [
  {
    selector: 'default',
    format: ['camelCase'],
  },
  {
    selector: 'variable',
    modifiers: ['const'],
    format: ['camelCase', 'PascalCase', 'UPPER_CASE'],
    leadingUnderscore: 'allow',
  },
  {
    selector: 'variableLike',
    format: ['camelCase'],
    leadingUnderscore: 'allow',
  },
  {
    selector: 'parameter',
    format: ['camelCase', 'PascalCase'],
    leadingUnderscore: 'allow',
  },
  {
    selector: 'function',
    format: ['camelCase', 'PascalCase'],
  },
  {
    selector: 'memberLike',
    format: ['camelCase'],
  },
  {
    selector: 'typeLike',
    format: ['PascalCase'],
  },
  {
    selector: 'property',
    format: null,
  },
];

const memberOrdering = {
  default: [
    'public-field',
    'protected-field',
    'private-field',

    'public-constructor',
    'protected-constructor',
    'private-constructor',

    'public-method',
    'protected-method',
    'private-method',
  ],
};

module.exports = {
  env: {
    es6: true,
    node: true,
  },
  parserOptions: {
    parser: '@typescript-eslint/parser',
    project: './tsconfig.json',
    tsconfigRootDir: __dirname,
    ecmaVersion: '2018',
    sourceType: 'module',
    ecmaFeatures: {
      jsx: true,
    },
  },
  ignorePatterns: ['test/*', 'scripts/*'],
  plugins: ['prettier', 'simple-import-sort'],
  extends: [
    'eslint:recommended',
    'plugin:@typescript-eslint/recommended',
    'plugin:react/recommended',
    'plugin:react/jsx-runtime',
  ],
  settings: {
    react: {
      createClass: 'createReactClass',
      pragma: 'React',
      version: 'detect',
    },
  },
  rules: {
    quotes: ['error', 'single', { avoidEscape: true }],
    'prettier/prettier': 'error',
    '@typescript-eslint/no-unused-vars': [
      'error',
      { argsIgnorePattern: '^_', ignoreRestSiblings: true },
    ],
    '@typescript-eslint/require-await': 'error',
    '@typescript-eslint/no-floating-promises': 'error',
    '@typescript-eslint/no-unused-expressions': 'error',
    '@typescript-eslint/member-ordering': ['error', memberOrdering],
    'no-return-await': 'error',
    'react/jsx-no-bind': 'error',
    '@typescript-eslint/naming-convention': ['error', ...namingConvention],
    '@typescript-eslint/ban-ts-comment': ['error', { 'ts-ignore': false }],
    'simple-import-sort/imports': 'error',

    '@typescript-eslint/no-use-before-define': 'off',
    '@typescript-eslint/explicit-module-boundary-types': 'off',
    '@typescript-eslint/no-non-null-assertion': 'off',
    'react/prop-types': 'off'
  },
};
