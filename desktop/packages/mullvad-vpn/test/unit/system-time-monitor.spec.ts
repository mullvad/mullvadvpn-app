import { expect, spy } from 'chai';
import sinon from 'sinon';

import { systemTimeMonitor } from '../../src/main/system-time-monitor';

describe('IAccountData cache', () => {
  let clock: sinon.SinonFakeTimers;

  beforeEach(() => {
    clock = sinon.useFakeTimers({ shouldAdvanceTime: true });
  });

  afterEach(() => {
    clock.restore();
  });

  it('should notify when system clock changes', () => {
    const systemTimeListener = spy();

    clock.setSystemTime(new Date('2025-01-01'));
    systemTimeMonitor(systemTimeListener);
    clock.setSystemTime(new Date('2025-01-02'));
    clock.tick(1001);
    clock.setSystemTime(new Date('2025-01-01'));
    clock.tick(1001);

    expect(systemTimeListener).to.have.been.called.twice;
  });
});
