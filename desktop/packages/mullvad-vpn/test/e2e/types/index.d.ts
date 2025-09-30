import '@playwright/test';

declare global {
  namespace PlaywrightTest {
    interface Matchers<R> {
      toMatchPath(template: string | null): Promise<R>;
    }
  }
}
