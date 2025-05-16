import react from 'eslint-plugin-react';
import reactcompiler from 'eslint-plugin-react-compiler';
import reactHooks from 'eslint-plugin-react-hooks';
import globals from 'globals';

import workspaceConfig from '../../eslint.config.mjs';

export default [
  ...workspaceConfig,
  react.configs.flat.recommended,
  { ignores: ['build/', 'build-standalone/'] },
  {
    files: ['**/*'],
    ignores: ['src/renderer/'],
    languageOptions: { globals: globals.node },
  },
  {
    files: ['src/renderer/'],
    languageOptions: { globals: globals.browser },
  },
  {
    files: ['test/'],
    languageOptions: { globals: globals.mocha },
  },
  {
    settings: {
      react: {
        createClass: 'createReactClass',
        pragma: 'React',
        version: 'detect',
      },
    },
  },
  {
    files: ['**/*.{js,mjs,ts,tsx}'],
    plugins: {
      'react-hooks': reactHooks,
      'react-compiler': reactcompiler,
    },
    rules: {
      'react/jsx-no-bind': 'error',
      'react-hooks/rules-of-hooks': 'error',
      'react-hooks/exhaustive-deps': 'error',
      'react-compiler/react-compiler': 'error',
      'react/prop-types': 'off',
      'react/react-in-jsx-scope': 'off',
    },
  },
  {
    files: ['test/**/*.spec.ts'],
    rules: { '@typescript-eslint/no-unused-expressions': 'off' },
  },
  {
    files: ['tasks/*', 'scripts/*'],
    rules: { '@typescript-eslint/no-require-imports': 'off' },
  },
];
