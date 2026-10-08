import workspaceConfig from '../../../desktop/eslint.config.mjs';

export default [
  ...workspaceConfig,
  {
    files: ['**/*.ts'],
    languageOptions: {
      parserOptions: {
        project: './tsconfig.eslint.json',
        tsconfigRootDir: import.meta.dirname,
      },
    },
  },
  {
    files: ['src/**/*.ts', '__tests__/**/*.ts', '*.config.ts'],
  },
  {
    ignores: ['dist/**', 'node_modules/**'],
  },
];
