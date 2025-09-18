import eslintReact from '@eslint-react/eslint-plugin';
import reactNamingConvention from 'eslint-plugin-react-naming-convention';
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
  // importPlugin.flatConfigs.typescript,
  // perfectionist.configs['recommended-alphabetical'],
  // eslintReact.configs['off'],
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
      // '@typescript-eslint/consistent-type-imports': [
      //   'error',
      //   {
      //     fixStyle: 'inline-type-imports',
      //     prefer: 'type-imports',
      //   },
      // ],
      // 'import/no-cycle': 'error',
      // 'perfectionist/sort-exports': [
      //   'error',
      //   {
      //     type: 'alphabetical',
      //     order: 'asc',
      //   },
      // ],
      'react-compiler/react-compiler': 'error',
      'react-hooks/exhaustive-deps': 'error',
      'react-hooks/rules-of-hooks': 'error',
      'react/jsx-no-bind': 'error',
      'react/prop-types': 'off',
      'react/react-in-jsx-scope': 'off',
    },
  },
  // {
  //   files: ['src/renderer/**/*.{ts,tsx}'],
  //   rules: {
  //     // Allow function statement and allow use of arrow functions in the file but
  //     // only allow export of function statement
  //     'func-style': [
  //       'error',
  //       'declaration',
  //       {
  //         allowArrowFunctions: true,
  //         overrides: {
  //           namedExports: 'declaration',
  //         },
  //       },
  //     ],
  //     // Only allow named exports
  //     'no-restricted-exports': [
  //       'error',
  //       {
  //         restrictDefaultExports: {
  //           direct: true, // Do not allow default exports
  //           named: true, // Do not allow exporting named as default
  //           defaultFrom: true, // Do not allow re-exporting default
  //           namedFrom: true, // Do not allow re-exporting named as default
  //           namespaceFrom: true, // Do not allow re-exporting * as default
  //         },
  //       },
  //     ],
  //   },
  // },
  // {
  //   files: ['src/renderer/**/*.tsx'],
  //   plugins: {
  //     '@eslint-react/naming-convention': reactNamingConvention,
  //   },
  //   rules: {
  //     // Enforce naming standard of React Components
  //     '@eslint-react/naming-convention/filename': ['error', { rule: 'PascalCase' }],
  //     // Use .tsx or .ts extension as needed
  //     '@eslint-react/naming-convention/filename-extension': ['error', 'as-needed'],
  //     // Sort a React Component's props alphabetically
  //     'react/jsx-sort-props': 'error',
  //   },
  // },
  // {
  //   files: ['src/renderer/**/hooks/use*.{ts}'],
  //   plugins: {
  //     '@eslint-react/naming-convention': reactNamingConvention,
  //   },
  //   rules: {
  //     // Enforce naming standard of React hooks
  //     '@eslint-react/naming-convention/filename': ['error', { rule: 'kebab-case' }],
  //   },
  // },
  {
    files: ['src/renderer/**/*.{ts,tsx}'],
    rules: {
      'no-restricted-imports': [
        'error',
        {
          // Allow imports only from index file of a barrel folder
          patterns: [
            // Forbidden to import from barrel folder children directly
            '**/components/*',
            '**/hooks/*',
            '**/utils/*',
            // Allowed to import from barrel folder index
            '!./components',
            '!./hooks',
            '!./utils',
            // Design system component library
            '!**/lib/components/*',
          ],
        },
      ],
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
