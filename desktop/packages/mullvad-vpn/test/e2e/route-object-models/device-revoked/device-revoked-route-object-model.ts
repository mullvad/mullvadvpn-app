import { RoutePath } from '../../../../src/shared/routes';
import { TestUtils } from '../../utils';

export class DeviceRevokedRouteObjectModel {
  constructor(private readonly utils: TestUtils) {}

  async waitForRoute() {
    await this.utils.expectRoute(RoutePath.deviceRevoked);
  }
}
