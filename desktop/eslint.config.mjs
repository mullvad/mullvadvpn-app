import eslint from '@eslint/js';
import prettier from 'eslint-plugin-prettier/recommended';
import simpleImportSort from 'eslint-plugin-simple-import-sort';
import tseslint from 'typescript-eslint';

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
    selector: 'import',
    format: ['camelCase', 'PascalCase', 'snake_case'],
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
    selector: 'typeProperty',
    format: ['camelCase'],
    filter: {
      regex: '^(data-testid|aria-labelledby|aria-describedby)$',
      match: false,
    },
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

export default tseslint.config(
  eslint.configs.recommended,
  ...tseslint.configs.recommended,
  prettier,
  {
    files: ['**/*.{ts,tsx}'],
    languageOptions: {
      parserOptions: {
        parser: '@typescript-eslint/parser',
        project: './tsconfig.json',
        ecmaVersion: '2018',
        sourceType: 'module',
        ecmaFeatures: {
          jsx: true,
        },
      },
    },
    rules: {
      '@typescript-eslint/require-await': 'error',
      '@typescript-eslint/no-floating-promises': 'error',

      '@typescript-eslint/no-use-before-define': 'off',
      '@typescript-eslint/explicit-module-boundary-types': 'off',
      '@typescript-eslint/no-non-null-assertion': 'off',
    },
  },
  {
    files: ['**/*.{js,mjs,ts,tsx}'],
    plugins: {
      'simple-import-sort': simpleImportSort,
    },
    rules: {
      quotes: ['error', 'single', { avoidEscape: true }],
      'prettier/prettier': 'error',
      '@typescript-eslint/no-unused-vars': [
        'error',
        { argsIgnorePattern: '^_', ignoreRestSiblings: true },
      ],
      '@typescript-eslint/no-unused-expressions': 'error',
      '@typescript-eslint/member-ordering': ['error', memberOrdering],
      'no-return-await': 'error',
      '@typescript-eslint/naming-convention': ['error', ...namingConvention],
      '@typescript-eslint/ban-ts-comment': 'error',
      'simple-import-sort/imports': 'error',
    },
  },
);
