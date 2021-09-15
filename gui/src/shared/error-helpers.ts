export function isErrorWithStringCode(maybeError: unknown): maybeError is Error & { code: string } {
  const error = maybeError as Error & { code: string };
  return error instanceof Error && 'code' in error;
}
