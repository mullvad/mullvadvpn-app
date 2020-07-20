function pascalCaseToCamelCaseImpl(str: string): string {
  return str.charAt(0).toLowerCase() + str.slice(1);
}

function snakeCaseToCamelCaseImpl(str: string): string {
  return str.replace(/_([a-z])/gi, (matches) => matches[1].toUpperCase());
}

function camelCaseToSnakeCaseImpl(str: string): string {
  return str
    .replace(/[a-z0-9][A-Z]/g, (matches) => `${matches[0]}_${matches[1].toLowerCase()}`)
    .toLowerCase();
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function pascalCaseToCamelCase<T>(anObject: { [key: string]: any }): T {
  return transformObjectKeys(anObject, pascalCaseToCamelCaseImpl) as T;
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function snakeCaseToCamelCase<T>(anObject: { [key: string]: any }): T {
  return transformObjectKeys(anObject, snakeCaseToCamelCaseImpl) as T;
}

export function camelCaseToSnakeCase<T>(anObject: T): Record<string, unknown> {
  return transformObjectKeys(anObject, camelCaseToSnakeCaseImpl);
}

function transformObjectKeys(
  anObject: { [key: string]: any }, // eslint-disable-line @typescript-eslint/no-explicit-any
  keyTransformer: (key: string) => string,
) {
  for (const sourceKey of Object.keys(anObject)) {
    const targetKey = keyTransformer(sourceKey);
    const sourceValue = anObject[sourceKey];

    anObject[targetKey] =
      sourceValue !== null && typeof sourceValue === 'object'
        ? transformObjectKeys(sourceValue, keyTransformer)
        : sourceValue;

    if (sourceKey !== targetKey) {
      delete anObject[sourceKey];
    }
  }
  return anObject;
}
