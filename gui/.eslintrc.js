module.exports = {
  env: {
    es6: true,
    node: true,
  },
  parserOptions: {
    parser: '@typescript-eslint/parser',
    project: './tsconfig.json',
    ecmaVersion: '2018',
    sourceType: 'module',
    ecmaFeatures: {
      jsx: true,
    },
  },
  ignorePatterns: ['test/*', 'scripts/*'],
  plugins: ['prettier'],
  extends: [
    'eslint:recommended',
    'plugin:@typescript-eslint/recommended',
    'plugin:react/recommended',
    'plugin:import/errors',
    'plugin:import/warnings',
    'plugin:import/typescript',
    'plugin:promise/recommended',
  ],
  settings: {
    react: {
      createClass: 'createReactClass',
      pragma: 'React',
      version: 'detect',
    },
  },
  rules: {
    'prettier/prettier': 'error',
    '@typescript-eslint/no-unused-vars': [
      'error',
      { argsIgnorePattern: '^_', ignoreRestSiblings: true },
    ],

    '@typescript-eslint/no-use-before-define': 'off',
    '@typescript-eslint/explicit-function-return-type': 'off',
    // TODO: Enable these
    'require-await': 'off',
    '@typescript-eslint/no-floating-promises': 'off',
    // TODO: This should eventually be removed.
    '@typescript-eslint/interface-name-prefix': 'off',
    // TODO: The rules below should be enabled when move from ReactXP is completed.
    '@typescript-eslint/camelcase': 'off',
    '@typescript-eslint/ban-ts-ignore': 'off',
    'react/no-find-dom-node': 'off',
  },
};
