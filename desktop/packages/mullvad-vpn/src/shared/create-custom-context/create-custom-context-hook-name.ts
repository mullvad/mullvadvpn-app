import camelCase from 'lodash/camelCase';
import snakeCase from 'lodash/snakeCase';

const createCustomContextHookName = (key: string) => {
  const snakeCaseNonPrefixedName = snakeCase(key);
  const snakeCasePrefixedName = `use_${snakeCaseNonPrefixedName}`;
  const camelCasePrefixedName = camelCase(snakeCasePrefixedName);

  const customContextHookName = camelCasePrefixedName;

  return customContextHookName;
};

export default createCustomContextHookName;
