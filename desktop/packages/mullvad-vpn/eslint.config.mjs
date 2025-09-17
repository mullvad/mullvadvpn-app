import importPlugin from 'eslint-plugin-import';
import perfectionist from 'eslint-plugin-perfectionist';
import react from 'eslint-plugin-react';
import reactcompiler from 'eslint-plugin-react-compiler';
import reactHooks from 'eslint-plugin-react-hooks';
import globals from 'globals';

import workspaceConfig from '../../eslint.config.mjs';

export default [
  ...workspaceConfig,
  react.configs.flat.recommended,
  importPlugin.flatConfigs.typescript,
  perfectionist.configs['recommended-alphabetical'],
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
      '@typescript-eslint/consistent-type-imports': [
        'error',
        {
          fixStyle: 'inline-type-imports',
          prefer: 'type-imports',
        },
      ],
      'import/no-cycle': 'error',
      'perfectionist/sort-exports': [
        'error',
        {
          type: 'alphabetical',
          order: 'asc',
        },
      ],
      'react-compiler/react-compiler': 'error',
      'react-hooks/exhaustive-deps': 'error',
      'react-hooks/rules-of-hooks': 'error',
      'react/jsx-no-bind': 'error',
      'react/jsx-sort-props': 'error',
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
